use color_eyre::{
    Section,
    eyre::{Context, ContextCompat},
};
use colored::Colorize;
use fs_err::tokio::{File, create_dir_all, hard_link};
use icns::{IconFamily, IconType, PixelFormat};
use image::{DynamicImage, ImageReader, imageops::FilterType};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::warn;

use crate::{commands::build::Builder, toml_conf::KaledisConfig};

fn resize_to_icns(img: &DynamicImage, size: u32) -> icns::Image {
    let resized = img
        .resize_exact(size, size, FilterType::Lanczos3)
        .to_rgba8();

    icns::Image::from_data(PixelFormat::RGBA, size, size, resized.into_raw())
        .expect("failed to convert image")
}

pub async fn build_macos(builder: &Builder, data: &[u8]) -> color_eyre::Result<()> {
    if builder.config.mac.is_none() {
        warn!("No valid macos config, skipping macos build");
        return Ok(());
    }

    println!(
        "{}", "WARNING: only unsigned builds are available for now. i don't have an mac. If you want to publish it officially i recommend using https://github.com/love2d/love/actions/".yellow()
    );

    let dists = builder.paths.dist.join("Macos");
    let contents = dists.join("love.app").join("Contents");
    let resources = contents.join("Resources");

    if let Some(icon) = &builder.config.icon {
        let img = ImageReader::open(builder.paths.root.join(icon))?.decode()?;
        let mut family = IconFamily::new();

        // ic04 → 16x16
        let img16 = resize_to_icns(&img, 16);
        family.add_icon_with_type(&img16, IconType::RGBA32_16x16)?;

        // ic11 → 32x32 (retina 16)
        let img32 = resize_to_icns(&img, 32);
        family.add_icon_with_type(&img32, IconType::RGBA32_16x16_2x)?;

        // ic07 → 128x128
        let img128 = resize_to_icns(&img, 128);
        family.add_icon_with_type(&img128, IconType::RGBA32_128x128)?;

        // ic13 → 256x256 (retina 128)
        let img256 = resize_to_icns(&img, 256);
        family.add_icon_with_type(&img256, IconType::RGBA32_128x128_2x)?;

        let file = std::io::BufWriter::new(
            std::fs::File::create(resources.join("OSXAppIcon2.icns")).unwrap(),
        );
        family.write(file).unwrap();
    }

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
            .context("Building for macos")?
            .filter_map(Result::ok)
        {
            let output_path = resources.join(
                path.strip_prefix(&builder.paths.root)
                    .context("Building for macos")
                    .suggestion("Don't use assets outside the root of your project")
                    .expect("Failed to strip root"),
            );
            create_dir_all(output_path.parent().unwrap()).await?;
            hard_link(&path, output_path).await?;
        }
    }

    create!(
        resources.join(format!("{}.love", &builder.config.project_name)),
        data
    );

    let plist_path = contents.join("Info.plist");
    let data = {
        let mut plist_file = File::open(&plist_path).await?;
        rewrite_app_files(&builder.config, &mut plist_file).await?
    };

    create!(plist_path, data.as_bytes());

    Ok(())
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
