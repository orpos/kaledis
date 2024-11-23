use std::{path::PathBuf, process::ExitCode, str::FromStr};

use anyhow::Result;
use clap::Parser;
use dal_core::{manifest::Manifest, polyfill::Polyfill, transpiler::Transpiler};
use url::Url;

use super::{DEFAULT_MANIFEST_PATH, DEFAULT_POLYFILL_URL};

/// Transpile luau files into lua files
#[derive(Debug, Clone, Parser)]
pub struct TranspileCommand {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
}

impl TranspileCommand {
    pub async fn run(self) -> Result<ExitCode> {
        let manifest = Manifest::from_file(DEFAULT_MANIFEST_PATH).await?;
        let polyfill = Polyfill::new(&Url::from_str(DEFAULT_POLYFILL_URL)?).await?;
        let mut transpiler = Transpiler::default();
        transpiler = transpiler.with_manifest(&manifest);
        transpiler = transpiler.with_polyfill(polyfill);

        transpiler
            .process(
                manifest.require_input(self.input)?,
                manifest.require_output(self.output)?,
            )
            .await?;

        return Ok(ExitCode::SUCCESS);
    }
}
