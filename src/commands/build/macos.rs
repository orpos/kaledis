use color_eyre::{
    Section,
    eyre::{Context, ContextCompat},
};
use colored::Colorize;
use fs_err::tokio::{File, create_dir_all, hard_link};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{commands::build::Builder, toml_conf::KaledisConfig};

pub async fn build_macos(builder: &Builder, data: &[u8]) {
    println!(
        "{}", "WARNING: only unsigned builds are available for now. i don't have an mac. If you want to publish it officially i recommend using https://github.com/love2d/love/actions/".yellow()
    );

    let dists = builder.paths.dist.join("Macos");
    let contents = dists.join("love.app").join("Contents");
    let resources = contents.join("Resources");

    create_dir_all(&dists)
        .await
        .expect("Failed to create macos dist folder");

    macro_rules! create {
        ($name: expr, $value :expr) => {{
            let mut f = File::create($name).await.expect("Failed to create file");
            f.write_all(&$value).await.expect("Failed to write files");
        }};
    }

    for pattern in &builder.config.layout.external {
        for path in glob::glob(&builder.paths.root.join(pattern).to_string_lossy())
            .context("Building for macos")
            .expect("Failed to parse glob")
            .filter_map(Result::ok)
        {
            let output_path = resources.join(
                path.strip_prefix(&builder.paths.root)
                    .context("Building for macos")
                    .suggestion("Don't use assets outside the root of your project")
                    .expect("Failed to strip root"),
            );
            create_dir_all(output_path.parent().unwrap())
                .await
                .expect("Failed to create assets folder");
            hard_link(&path, output_path)
                .await
                .expect("Failed to link file");
        }
    }

    create!(
        resources.join(format!("{}.love", &builder.config.project_name)),
        data
    );

    let plist_path = contents.join("Info.plist");
    let data = {
        let mut plist_file = File::open(&plist_path)
            .await
            .expect("Failed to open plist file");
        rewrite_app_files(&builder.config, &mut plist_file)
            .await
            .expect("Failed to process plist")
    };

    create!(plist_path, data.as_bytes());
}

// Credit: https://github.com/camchenry/boon

/// Rewrites the macOS application files to contain the project's info
async fn rewrite_app_files(config: &KaledisConfig, file: &mut File) -> color_eyre::Result<String> {
    let mac = config
        .mac
        .as_ref()
        .wrap_err("No Mac manifest in kaledis.toml")
        .suggestion("Try adding the mac field on the manifest")?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).await?;
    let re = regex::Regex::new("(CFBundleIdentifier.*\n\t<string>)(.*)(</string>)")
        .context("Failed to create regex")?;
    buffer = re
        .replace(buffer.as_str(), |caps: &regex::Captures| {
            [
                caps.get(1).expect("Could not get capture").as_str(),
                mac.id.as_str(),
                caps.get(3).expect("Could not get capture").as_str(),
            ]
            .join("")
        })
        .to_string();
    let re = regex::Regex::new("(CFBundleName.*\n\t<string>)(.*)(</string>)")
        .context("Could not create regex")?;
    buffer = re
        .replace(buffer.as_str(), |caps: &regex::Captures| {
            [
                caps.get(1).expect("Could not get capture").as_str(),
                config.project_name.as_str(),
                caps.get(3).expect("Could not get capture").as_str(),
            ]
            .join("")
        })
        .to_string();
    let re = regex::RegexBuilder::new("^\t<key>UTExportedTypeDeclarations.*(\n.*)+\t</array>\n")
        .multi_line(true)
        .build()
        .context("Could not build regex")?;
    buffer = re.replace(buffer.as_str(), "").to_string();
    Ok(buffer)
}
