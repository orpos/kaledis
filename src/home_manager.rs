use clap::ValueEnum;
use color_eyre::eyre::Context;
use dirs::home_dir;
use fs_err::tokio::{self as fs, File};
use reqwest::Client;
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
static APKTOOL_HASH: &str = "66cf4524a4a45a7f56567d08b2c9b6ec237bcdd78cee69fd4a59c8a0243aeafa";

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

        if let Err(_) = fs::write(
            kaledis_dir.join("globals.d.luau"),
            include_bytes!("../static/globals.d.luau"),
        )
        .await
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
        let pth = self.path.join(version).join(platform.as_ref().to_string());
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
        let hash = hex_literal::hex!("db7fe2f05857074e73ef2bb10bfb95556ad110cf1ba0c82d101f93b3a93862ff");

        #[cfg(target_os = "linux")]
        let url = "https://github.com/adoptium/temurin11-binaries/releases/download/jdk-11.0.30%2B7/OpenJDK11U-jre_aarch64_linux_hotspot_11.0.30_7.tar.gz";
        #[cfg(target_os = "linux")]
        let hash = hex_literal::hex!("9d6a8d3a33c308bbc7332e4c2e2f9a94fbbc56417863496061ef6defef9c5391");
        
        #[cfg(target_os = "macos")]
        let url = "https://github.com/adoptium/temurin11-binaries/releases/download/jdk-11.0.30%2B7/OpenJDK11U-jdk_aarch64_mac_hotspot_11.0.30_7.tar.gz";
        #[cfg(target_os = "macos")]
        let hash = hex_literal::hex!("d7b52d25d6f7aae2d4d85191d84bc132b80d061006dcd5f76ca79f277c3acb28");

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
                create_dir_all(&jv).await?.context("Extracting the zip")?;

                println!("A");

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
        let exe_name = match platform {
            Target::LoveFile => "".to_string(),
            Target::Android => format!("love-{}-android.apk", version),
            Target::LinuxAppImage => format!("love-{}-x86_64.AppImage", version),
            Target::Macos => format!("love-{}-macos.zip", version),
            Target::Windows => format!("love-{}-win64.zip", version),
        };

        let output_version = self
            .path
            .join(&version)
            .join(&platform.as_ref().to_string());

        if output_version.exists() {
            return;
        }

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
                return;
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
