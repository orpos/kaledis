use std::io::Write;

use anyhow::Context;
use reqwest::header::ACCEPT;
use semver::Version;
use serde::Deserialize;
use tokio::process::Command;

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    url: url::Url,
}

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

fn get_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION").trim_start_matches("v")).unwrap()
}

pub async fn get_latest_remote_version(reqwest: &reqwest::Client) -> Version {
    let releases = reqwest
        .get("https://api.github.com/repos/orpos/kaledis/releases")
        .send().await
        .context("Failed to send request to Github API")
        .unwrap()
        .json::<Vec<Release>>().await
        .unwrap();
    releases
        .into_iter()
        .map(|release| Version::parse(&release.tag_name.trim_start_matches("v")))
        .filter_map(Result::ok)
        .max()
        .unwrap_or(get_version())
}

pub async fn check_for_updates(reqwest: &reqwest::Client) {
    let latest = get_latest_remote_version(reqwest).await;
    if latest >= get_version() {
        println!("New update found! Updating...");
        let release = reqwest
            .get(format!("https://api.github.com/repos/orpos/kaledis/releases/tags/v{latest}"))
            .send().await
            .unwrap()
            .json::<Release>().await
            .unwrap();
        let asset = release.assets.into_iter().next().unwrap();
        let bytes = reqwest
            .get(asset.url)
            .header(ACCEPT, "application/octet-stream")
            .send().await
            .unwrap()
            .bytes().await
            .unwrap();
        // TODO: make the release files be a zip and support other platforms
        let local_path = std::env::current_exe().unwrap();
        {
            let mut new_exe = std::fs::File::create(local_path.with_file_name("new.exe")).unwrap();
            new_exe.write(&bytes).unwrap();
        }
        Command::new(local_path.with_file_name("new.exe"))
            .args(vec!["__new__", &local_path.display().to_string()])
            .spawn()
            .unwrap();
    } else {
        println!("No update found.")
    }
}

pub async fn update() {
    let reqwest = {
        let mut headers = reqwest::header::HeaderMap::new();

        headers.insert(
            reqwest::header::ACCEPT,
            "application/json".parse().context("failed to create accept header").unwrap()
        );

        reqwest::Client
            ::builder()
            .user_agent(concat!("kaledis", "/", "updater"))
            .default_headers(headers)
            .build()
            .unwrap()
    };
    check_for_updates(&reqwest).await;
}
