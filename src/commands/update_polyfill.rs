use std::process::ExitCode;

use kaledis_dalbit::manifest::Manifest;

use crate::toml_conf::Config;

pub async fn update_polyfill() -> anyhow::Result<ExitCode> {
    let cfg = Config::from_toml_file("kaledis.toml")?;
    let mut manifest = Manifest::default();
    if let Some(polyfill) = cfg.polyfill {
        manifest.polyfill = Some(polyfill.polyfill().await.unwrap());
    }
    let Some(polyfill) = manifest.polyfill() else {
        println!("No polyfill configured in the manifest.");
        return Ok(ExitCode::SUCCESS);
    };
    let polyfill_cache = polyfill.cache().await?;
    polyfill_cache.fetch()?;

    println!("Fetched new polyfill");

    return Ok(ExitCode::SUCCESS);
}
