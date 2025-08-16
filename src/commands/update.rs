use std::env::temp_dir;
use std::io::Write;

use anyhow::Context;
use futures::StreamExt;
use reqwest::header::ACCEPT;
use semver::Version;
use serde::Deserialize;
use tokio::process::Command;

use tokio::io::AsyncReadExt;

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

fn get_repo() -> (String, String) {
    let mut parts = env!("CARGO_PKG_REPOSITORY").split('/').skip(3);
    (
        parts.next().unwrap().to_string(),
        parts.next().unwrap().to_string(),
    )
}

fn get_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION").trim_start_matches("v")).unwrap()
}

pub async fn get_latest_remote_version(reqwest: &reqwest::Client) -> anyhow::Result<Version> {
    let (owner, repo) = get_repo();
    let releases = reqwest
        .get(format!(
            "https://api.github.com/repos/{owner}/{repo}/releases"
        ))
        .send()
        .await
        .context("Failed to send request to GitHub API")
        .unwrap()
        .json::<Vec<Release>>()
        .await
        .unwrap();

    releases
        .into_iter()
        .map(|release| Version::parse(&release.tag_name.trim_start_matches("v")))
        .filter_map(Result::ok)
        .max()
        .context("Failed to find first version.")
}

pub async fn get_update(reqwest: &reqwest::Client, allow_breaking: bool) -> anyhow::Result<bool> {
    let latest = get_latest_remote_version(reqwest).await?;
    if latest > get_version() {
        if latest.major > get_version().major && !allow_breaking {
            eprintln!("Major update detected. Add --allow-breaking flag to update");
            return Ok(false);
        }
        println!("New update found! Updating...");

        let release = reqwest
            .get(format!(
                "https://api.github.com/repos/orpos/kaledis/releases/tags/v{}",
                latest
            ))
            .send()
            .await
            .unwrap()
            .json::<Release>()
            .await
            .unwrap();

        let asset = release
            .assets
            .into_iter()
            .find(|asset| {
                asset.name.ends_with(&format!(
                    "-{}-{}.tar.gz",
                    std::env::consts::OS,
                    std::env::consts::ARCH
                ))
            })
            .context("Failed to find a version for current platform")?;
        let bytes = reqwest
            .get(asset.url)
            .header(ACCEPT, "application/octet-stream")
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();

        let mut decoder = async_compression::tokio::bufread::GzipDecoder::new(bytes.as_ref());
        let mut archive = tokio_tar::Archive::new(&mut decoder);

        let mut entry = archive
            .entries()
            .context("Failed to read archive")?
            .next()
            .await
            .context("Archive has no files.")?
            .context("Failed to get first file")?;

        let mut buffer = Vec::new();

        entry
            .read_to_end(&mut buffer)
            .await
            .context("Failed to read the bytes.")?;

        let exe = temp_dir().with_file_name("new.exe");

        {
            let mut new_exe = std::fs::File::create(&exe).unwrap();
            new_exe.write(&buffer).unwrap();
        }
        Command::new(&exe)
            .args(vec!["update", &temp_dir().display().to_string()])
            .spawn()
            .unwrap();
        std::process::exit(0);
    } else {
        println!("No update found.");
        return Ok(false);
    }
}

pub async fn update(allow_breaking: bool) {
    let reqwest = {
        let mut headers = reqwest::header::HeaderMap::new();

        headers.insert(
            reqwest::header::ACCEPT,
            "application/json"
                .parse()
                .context("failed to create accept header")
                .unwrap(),
        );

        reqwest::Client::builder()
            .user_agent(concat!("kaledis", "/", "updater"))
            .default_headers(headers)
            .build()
            .unwrap()
    };
    get_update(&reqwest, allow_breaking).await.unwrap();
}
