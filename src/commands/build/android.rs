use fs_err::tokio::{File, create_dir_all, remove_dir_all};
use image::{GenericImageView, imageops::FilterType};
use tokio::{io::AsyncWriteExt, process::Command};
use tracing::info;

use crate::{commands::build::Builder, home_manager::Target};

macro_rules! create {
    ($name: expr, $value :expr) => {{
        let mut f = File::create($name).await.expect("Failed to create file");
        f.write_all(&$value).await.expect("Failed to write files");
    }};
    (ensure_path => $name: expr) => {{
        create_dir_all($name)
            .await
            .expect("Failed to create folder");
    }};
    (remove => $name: expr) => {{
        remove_dir_all($name)
            .await
            .expect("Failed to create folder");
    }};
}
pub async fn build_android(builder: &Builder, data: &[u8]) -> color_eyre::Result<()> {
    tracing::warn!("Remember to sign your android build");
    let Some(config) = &builder.config.android else {
        eprintln!("No valid android config, skipping android build...");
        tracing::warn!("No valid android config, skipping android build...");
        return Ok(());
    };

    let dist_folder = builder.paths.dist.join("Android");
    let project_name = &builder.config.project_name;
    let love_version = &builder.config.love;
    let home = &builder.home;

    home.ensure_java().await?;
    home.ensure_apktool().await?;

    let apk = home
        .get_path(love_version, Target::Android)
        .await
        .join("love2d.apk");

    let apktool_jar = home.get_apktool_path();
    let java = home.get_java_path();

    let build_folder = dist_folder.join("build");

    let apktool = async |args: &[&str]| {
        Command::new(&java)
            .args(
                ["-jar", &apktool_jar.to_string_lossy()]
                    .into_iter()
                    .chain(args.to_owned()),
            )
            .spawn()
            .expect("Failed to spawn apktool")
            .wait()
            .await
            .expect("Apktool failed")
    };

    println!("Unzipping apk file");

    apktool(&[
        "d",
        "-f",
        "-s",
        "-o",
        &build_folder.to_string_lossy(),
        &apk.to_string_lossy(),
    ])
    .await;

    if let Some(icon) = &builder.config.icon {
        let img = image::open(icon)?;

        for logo in glob::glob(
            &build_folder
                .join("res/**/love.png")
                .to_string_lossy()
                .to_string(),
        )?
        .filter_map(Result::ok)
        {
            let (width, height) = {
                let original_image = image::open(&logo)?;
                original_image.dimensions()
            };
            let resized = img
                .resize_exact(width, height, FilterType::Lanczos3)
                .to_rgba8();
            resized.save(logo)?;
        }
    }

    let android_manifest = build_folder.join("AndroidManifest.xml");

    // Creates AndroidManifest.xml
    create!(&android_manifest, config.to_string(project_name).as_bytes());

    let bundle = build_folder.join("assets").join("game.love");
    create!(ensure_path => &bundle.parent().unwrap());
    create!(&bundle, &data);

    info!("Building apk...");

    apktool(&[
        "b",
        "-o",
        &dist_folder.join("app.apk").to_string_lossy(),
        &build_folder.to_string_lossy(),
    ])
    .await;

    info!("Cleaning build folder...");

    create!(remove => &build_folder);

    Ok(())
}
