use crate::commands::build::Builder;
use fs_err::tokio::{File, remove_file};

pub async fn build_windows(builder: &Builder, data: &[u8]) -> anyhow::Result<()> {
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

    Ok(())
}
