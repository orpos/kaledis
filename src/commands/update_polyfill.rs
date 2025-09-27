use std::process::ExitCode;

use kaledis_dalbit::manifest::Manifest;

pub async fn update_polyfill() -> anyhow::Result<ExitCode> {
    let manifest = Manifest::default();
    let Some(polyfill) = manifest.polyfill() else {
        println!("No polyfill configured in the manifest.");
        return Ok(ExitCode::SUCCESS);
    };
    let polyfill_cache = polyfill.cache().await?;
    polyfill_cache.fetch()?;

    println!("Fetched new polyfill");

    return Ok(ExitCode::SUCCESS);
}
