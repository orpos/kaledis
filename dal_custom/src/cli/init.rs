use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;
use dal_core::manifest::Manifest;

use crate::cli::DEFAULT_MANIFEST_PATH;

/// Initialize dal manifest file
#[derive(Debug, Clone, Parser)]
pub struct InitCommand {}

impl InitCommand {
    pub async fn run(self) -> Result<ExitCode> {
        let manifest = Manifest::default();
        manifest.write(DEFAULT_MANIFEST_PATH).await?;

        println!("Initialized dal.toml");

        return Ok(ExitCode::SUCCESS);
    }
}
