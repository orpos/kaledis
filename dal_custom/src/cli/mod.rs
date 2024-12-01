use std::process::ExitCode;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

mod fetch;
mod init;
mod transpile;
mod clean;

use fetch::FetchCommand;
use init::InitCommand;
use log::LevelFilter;
use transpile::TranspileCommand;
use clean::CleanCommand;

pub const DEFAULT_POLYFILL_URL: &str = "https://github.com/CavefulGames/dal-polyfill";
pub const DEFAULT_MANIFEST_PATH: &str = "dal.toml";

#[derive(Debug, Clone, Subcommand)]
pub enum CliSubcommand {
    Transpile(TranspileCommand),
    Init(InitCommand),
    Fetch(FetchCommand),
	Clean(CleanCommand),
}

#[derive(Debug, Args, Clone)]
pub struct GlobalOptions {
    /// Sets verbosity level (can be specified multiple times)
    #[arg(long, short, global(true), action = clap::ArgAction::Count)]
    verbose: u8,
}

impl GlobalOptions {
    pub fn get_log_level_filter(&self) -> LevelFilter {
        match self.verbose {
            0 => LevelFilter::Warn,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    }
}

/// Transpile Luau scripts
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Dal {
	#[command(flatten)]
    global_options: GlobalOptions,
    #[clap(subcommand)]
    subcommand: CliSubcommand,
}

impl Dal {
    pub async fn run(self) -> Result<ExitCode> {
        match self.subcommand {
            CliSubcommand::Transpile(cmd) => cmd.run().await,
            CliSubcommand::Init(cmd) => cmd.run().await,
            CliSubcommand::Fetch(cmd) => cmd.run().await,
			CliSubcommand::Clean(cmd) => cmd.run().await,
        }
    }

	pub fn get_log_level_filter(&self) -> LevelFilter {
        self.global_options.get_log_level_filter()
    }
}
