use crate::commands::build::Builder;
use color_eyre::{Section, eyre::Context};
use fs_err::tokio::{File, create_dir_all, hard_link, remove_file};

#[tracing::instrument(skip(builder, data))]
pub async fn build_windows(builder: &Builder, data: &[u8]) -> color_eyre::Result<()> {
    let dists = builder.paths.dist.join("Windows");
    let exe = dists.join("love.exe");
    let mut exe_file = File::open(&exe).await.expect("Failed to open love.exe");

    let mut output = File::create(dists.join(&(builder.config.project_name.clone() + ".exe")))
        .await
        .expect("Failed to create final exe");

    tokio::io::copy(&mut exe_file, &mut output).await?;
    tokio::io::copy(&mut &data[..], &mut output).await?;

    remove_file(exe)
        .await
        .expect("Failed to remove original love.exe");

    for pattern in &builder.config.layout.external {
        for path in glob::glob(&builder.paths.root.join(pattern).to_string_lossy())
            .context("Building for windows")
            .expect("Failed to parse glob")
            .filter_map(Result::ok)
        {
            let output = dists.join(
                path.strip_prefix(&builder.paths.root)
                    .context("Building for windows")
                    .suggestion("Don't use assets outside the root of your project")
                    .expect("Failed to strip root"),
            );
            create_dir_all(&output.parent().unwrap())
                .await
                .expect("Failed to create output file");
            if output.exists() {
                remove_file(&output).await.expect("Failed to clean folder");
            }
            hard_link(&path, output).await.expect("Failed to link file");
        }
    }

    Ok(())
}
