use std::{process::ExitCode, str::FromStr};

use anyhow::Result;
use clap::Parser;
use dal_core::polyfill::Polyfill;
use url::Url;

use super::DEFAULT_POLYFILL_URL;

/// Fetch dal polyfills
#[derive(Debug, Clone, Parser)]
pub struct FetchCommand {}

impl FetchCommand {
    pub async fn run(self) -> Result<ExitCode> {
        let polyfill = Polyfill::new(&Url::from_str(DEFAULT_POLYFILL_URL)?).await?;
        polyfill.fetch()?;

        return Ok(ExitCode::SUCCESS);
    }
}
