use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod fetch;
mod init;
mod transpile;

use fetch::FetchCommand;
use init::InitCommand;
use transpile::TranspileCommand;

pub const DEFAULT_POLYFILL_URL: &str = "https://github.com/CavefulGames/dal-polyfill";
pub const DEFAULT_MANIFEST_PATH: &str = "dal.toml";

#[derive(Debug, Clone, Subcommand)]
pub enum CliSubcommand {
    Transpile(TranspileCommand),
    Init(InitCommand),
    Fetch(FetchCommand),
}

/// Transpile Luau scripts
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Dal {
    #[clap(subcommand)]
    subcommand: CliSubcommand,
}

impl Dal {
    pub fn new() -> Self {
        Self::parse()
    }

    pub async fn run(self) -> Result<ExitCode> {
        match self.subcommand {
            CliSubcommand::Transpile(cmd) => cmd.run().await,
            CliSubcommand::Init(cmd) => cmd.run().await,
            CliSubcommand::Fetch(cmd) => cmd.run().await,
        }
    }
}
