use clap::ValueEnum;
use color_eyre::eyre::Context;
use dirs::home_dir;
use fs_err::tokio::{self as fs, File};
use reqwest::Client;
use serde::Deserialize;
use sha2::Digest;
use std::{
    io::Cursor,
    path::{Path, PathBuf},
};
use strum_macros::AsRefStr;
use tokio::io::AsyncWriteExt;
use zip::ZipArchive;

static APKTOOL_LOCATION: &str =
    "https://github.com/iBotPeaches/Apktool/releases/download/v2.12.1/apktool_2.12.1.jar";
// Idk but, handling external binaries is something i want to do safely and sparingly
static APKTOOL_HASH: &[u8; 32] =
    &hex_literal::hex!("66cf4524a4a45a7f56567d08b2c9b6ec237bcdd78cee69fd4a59c8a0243aeafa");

#[cfg(windows)]
pub static CURRENT_PLATFORM: Target = Target::Windows;
#[cfg(target_os = "linux")]
pub static CURRENT_PLATFORM: Target = Target::LinuxAppImage;
#[cfg(target_os = "macos")]
pub static CURRENT_PLATFORM: Target = Target::Macos;

pub struct HomeManager {
    pub path: PathBuf,
    pub client: Client,
}

#[derive(AsRefStr, Debug, PartialEq, Eq, Clone, ValueEnum)]
pub enum Target {
    Windows,
    LinuxAppImage,
    Android,
    Macos,
    LoveFile,
}

impl HomeManager {
    pub async fn new() -> color_eyre::Result<Self> {
        let kaledis_dir = home_dir()
            .unwrap_or(
                dirs::preference_dir().expect("Failed to find a path to put versions and cache"),
            )
            .join(".kaledis");
        if !fs::try_exists(&kaledis_dir).await.unwrap_or(false) {
            fs::create_dir(&kaledis_dir)
                .await
                .context("Creating .kaledis folder")?;
            fs::create_dir(&kaledis_dir.join("versions"))
                .await
                .context("Creating version folder")?;
        }

        if fs::write(
            kaledis_dir.join("globals.d.luau"),
            include_bytes!("../static/globals.d.luau"),
        )
        .await
        .is_err()
        {
            eprintln!("Failed to create globals.d.luau file, resuming...");
            // todo: log error with debug flag
        };

        Ok(Self {
            path: kaledis_dir,
            client: Client::new(),
        })
    }

    pub async fn get_path(&self, version: &str, platform: Target) -> PathBuf {
        let pth = self.path.join(version).join(platform.as_ref());
        if let Target::Windows = platform {
            return pth.join(format!("love-{}-win64", version));
        }
        pth
    }

    pub fn get_java_path(&self) -> PathBuf {
        let mut path = self.path.join("java").join("jdk-11.0.30+7-jre").join("bin");
        #[cfg(windows)]
        path.push("java.exe");
        #[cfg(not(windows))]
        path.push("java");
        path
    }
    pub fn get_apktool_path(&self) -> PathBuf {
        self.path.join("java").join("tool.java")
    }

    pub async fn ensure_apktool(&self) -> color_eyre::Result<()> {
        let jv = self.path.join("java").join("tool.java");
        if jv.exists() {
            return Ok(());
        }

        let response = self.client.get(APKTOOL_LOCATION).send().await.unwrap();
        let bytes = response.bytes().await.unwrap();

        let result = sha2::Sha256::digest(&bytes);

        assert_eq!(result[..], *APKTOOL_HASH);

        let mut file = File::create(jv)
            .await
            .context("Creating apktool java file")?;
        file.write_all(&bytes)
            .await
            .context("Writing apktool java file")?;

        Ok(())
    }

    // jdk-11.0.30+7-jre
    pub async fn ensure_java(&self) -> color_eyre::Result<()> {
        #[cfg(windows)]
        let url = "https://github.com/adoptium/temurin11-binaries/releases/download/jdk-11.0.30%2B7/OpenJDK11U-jre_x64_windows_hotspot_11.0.30_7.zip";
        #[cfg(windows)]
        let hash =
            hex_literal::hex!("db7fe2f05857074e73ef2bb10bfb95556ad110cf1ba0c82d101f93b3a93862ff");

        #[cfg(target_os = "linux")]
        let url = "https://github.com/adoptium/temurin11-binaries/releases/download/jdk-11.0.30%2B7/OpenJDK11U-jre_aarch64_linux_hotspot_11.0.30_7.tar.gz";
        #[cfg(target_os = "linux")]
        let hash =
            hex_literal::hex!("9d6a8d3a33c308bbc7332e4c2e2f9a94fbbc56417863496061ef6defef9c5391");

        #[cfg(target_os = "macos")]
        let url = "https://github.com/adoptium/temurin11-binaries/releases/download/jdk-11.0.30%2B7/OpenJDK11U-jdk_aarch64_mac_hotspot_11.0.30_7.tar.gz";
        #[cfg(target_os = "macos")]
        let hash =
            hex_literal::hex!("d7b52d25d6f7aae2d4d85191d84bc132b80d061006dcd5f76ca79f277c3acb28");

        let jv = self.path.join("java");
        if jv.exists() {
            return Ok(());
        }

        let response = self.client.get(url).send().await.unwrap();

        let bytes = response.bytes().await.unwrap();
        let result = sha2::Sha256::digest(&bytes);
        // If this fails, the contents of the java runtime are different from when i got them.
        // So it's better to not run then.
        assert_eq!(result[..], hash);
        #[cfg(windows)]
        {
            use color_eyre::{Section, eyre::Context};
            use fs_err::tokio::create_dir_all;

            let jv = self.path.join("java");
            create_dir_all(&jv)
                .await
                .context("Creating java dir")
                .suggestion("Try removing ~/.kaledis/java folder")?;

            tokio::task::spawn_blocking(move || {
                extract_zip(Cursor::new(bytes), jv);
            })
            .await
            .context("Extracting the zip")?;
        }
        #[cfg(not(windows))]
        {
            use flate2::bufread::GzDecoder;
            use fs_err::tokio::create_dir_all;

            let jv = self.path.join("java");
            if !jv.exists() {
                create_dir_all(&jv).await.context("Extracting the zip")?;

                let decoder = GzDecoder::new(Cursor::new(bytes));
                let mut archive = tar::Archive::new(decoder);
                archive.unpack(jv).context("Extracting java folder")?;
            }
        }
        Ok(())
    }

    // Has to be like 11.5 | 11.3 etc
    // version 12 is only available when gh cli is available
    pub async fn ensure_version(&self, version: &str, platform: Target) {
        let output_version = self.path.join(version).join(platform.as_ref());

        if output_version.exists() {
            return;
        }

        if version.starts_with("12") {
            check_gh_available().expect(
                "Love2D version 12 requires the GitHub CLI (gh).\n\
                 Please install it from: https://cli.github.com\n\
                 After installing, run 'gh auth login' to authenticate.",
            );
            download_via_gh(platform, &output_version, version)
                .await
                .expect("Failed to download Love2D v12 via gh CLI");
            return;
        }

        let exe_name = match platform {
            Target::LoveFile => "".to_string(),
            Target::Android => format!("love-{}-android.apk", version),
            Target::LinuxAppImage => format!("love-{}-x86_64.AppImage", version),
            Target::Macos => format!("love-{}-macos.zip", version),
            Target::Windows => format!("love-{}-win64.zip", version),
        };

        let response = self
            .client
            .get(format!(
                "https://github.com/love2d/love/releases/download/{}/{}",
                version, exe_name
            ))
            .send()
            .await
            .unwrap();

        let bytes = response.bytes().await.unwrap();

        match platform {
            Target::Android | Target::LinuxAppImage => {
                std::fs::create_dir_all(&output_version).unwrap();
                let mut file = std::fs::File::create_new(output_version.join(
                    if let Target::Android = platform {
                        "love2d.apk"
                    } else {
                        "love2d.AppImage"
                    },
                ))
                .unwrap();
                std::io::copy(&mut Cursor::new(bytes), &mut file).unwrap();
            }
            _ => {
                tokio::task::spawn_blocking(move || {
                    extract_zip(Cursor::new(bytes), output_version);
                })
                .await
                .unwrap();
            }
        }
    }
}

fn check_gh_available() -> color_eyre::Result<()> {
    match std::process::Command::new("gh")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
    {
        Ok(status) if status.success() => Ok(()),
        _ => Err(color_eyre::eyre::eyre!(
            "GitHub CLI (gh) is not installed or not found in PATH.\n\
             Love2D version 12 requires the GitHub CLI to download CI artifacts.\n\
             Install it from: https://cli.github.com"
        )),
    }
}

#[derive(Deserialize)]
struct GhRun {
    #[serde(rename = "databaseId")]
    database_id: u64,
}

/// Returns the artifact name used in the love2d/love CI workflow for the given target.
fn gh_artifact_name(platform: &Target) -> &'static str {
    match platform {
        Target::Windows => "love-windows-x64",
        Target::LinuxAppImage => "love-linux-X64.AppImage",
        Target::Macos => "love-macos",
        Target::Android => "love-android.apk",
        Target::LoveFile => unreachable!("LoveFile does not need a download"),
    }
}

async fn download_via_gh(
    platform: Target,
    output_version: &PathBuf,
    version: &str,
) -> color_eyre::Result<()> {
    // 1. Get the latest successful run ID
    let run_output = tokio::process::Command::new("gh")
        .args([
            "run",
            "list",
            "--branch",
            "main",
            "--status",
            "success",
            "--limit",
            "1",
            "--repo",
            "love2d/love",
            "--json",
            "databaseId",
        ])
        .output()
        .await
        .context("Failed to run 'gh run list'")?;

    if !run_output.status.success() {
        return Err(color_eyre::eyre::eyre!(
            "'gh run list' failed: {}",
            String::from_utf8_lossy(&run_output.stderr)
        ));
    }

    let runs: Vec<GhRun> = serde_json::from_slice(&run_output.stdout)
        .context("Failed to parse 'gh run list' JSON output")?;

    let run = runs
        .first()
        .ok_or_else(|| color_eyre::eyre::eyre!("No successful CI runs found for love2d/love"))?;

    let run_id = run.database_id.to_string();
    let artifact_name = gh_artifact_name(&platform);

    println!(
        "Downloading Love2D v12 artifact '{}' from run {}...",
        artifact_name, run_id
    );

    std::fs::create_dir_all(output_version)
        .context("Failed to create output directory for Love2D v12")?;

    let download_output = tokio::process::Command::new("gh")
        .args([
            "run",
            "download",
            &run_id,
            "--name",
            artifact_name,
            "--repo",
            "love2d/love",
            "--dir",
            &output_version.to_string_lossy(),
        ])
        .output()
        .await
        .context("Failed to run 'gh run download'")?;

    if !download_output.status.success() {
        // Clean up the empty directory on failure
        let _ = std::fs::remove_dir_all(output_version);
        return Err(color_eyre::eyre::eyre!(
            "'gh run download' failed: {}",
            String::from_utf8_lossy(&download_output.stderr)
        ));
    }

    match platform {
        Target::Windows => {
            let zip_name = format!("love-{}-win64.zip", version);
            let zip_path = output_version.join(&zip_name);
            if zip_path.exists() {
                println!("Extracting {}...", zip_name);
                let bytes =
                    std::fs::read(&zip_path).context("Failed to read the downloaded zip")?;
                extract_zip(Cursor::new(bytes.into()), output_version.clone());
            }
        }
        Target::Macos => {
            let zip_path = output_version.join("love-macos.zip");
            if zip_path.exists() {
                println!("Extracting love-macos.zip...");
                let bytes =
                    std::fs::read(&zip_path).context("Failed to read the downloaded zip")?;
                extract_zip(Cursor::new(bytes.into()), output_version.clone());
            }
        }
        Target::LinuxAppImage => {
            for entry in std::fs::read_dir(output_version)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("AppImage") {
                    std::fs::rename(&path, output_version.join("love2d.AppImage"))?;
                    break;
                }
            }
        }
        Target::Android => {
            let apk_path = output_version.join("app-normal-record-release-unsigned.apk");
            if apk_path.exists() {
                std::fs::rename(apk_path, output_version.join("love2d.apk"))?;
            }
        }
        _ => {}
    }

    println!("Love2D v12 artifact downloaded successfully.");
    Ok(())
}

pub fn extract_zip(bytes: Cursor<tokio_util::bytes::Bytes>, output: PathBuf) {
    let mut archive = ZipArchive::new(bytes).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();

        let outpath = Path::new(&output).join(file.name());

        if file.is_dir() {
            std::fs::create_dir_all(&outpath).unwrap();
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }

            let mut outfile = std::fs::File::create(&outpath).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }
    }
}
