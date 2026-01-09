use std::process::ExitCode;
use crate::dalbit::manifest::Manifest;
use crate::dalbit::transpile::clean_polyfill;
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
    let polyfill_cache = polyfill.cache()?;
    polyfill_cache.fetch()?;
    clean_polyfill();

    println!("Fetched new polyfill");

    return Ok(ExitCode::SUCCESS);
}
