mod build;
pub mod init;
mod update_polyfill;
mod watch;
// mod update;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[clap(about = "Initializes a new Love2D project.")]
    Init { path: Option<PathBuf> },
    #[clap(
        about = "Transpiles everything, and builds a '.love' file inside a '.build' directory."
    )]
    Build {
        path: Option<PathBuf>,
        #[arg(short, long, help = "A config that joins all files in a single one.")]
        one_file: bool,
    },
    #[clap(about = "Compiles the entire project to a executable, inside a 'dist' folder.")]
    Compile {
        path: Option<PathBuf>,
        #[arg(short, long, help = "A config that joins all files in a single one.")]
        one_file: bool,
    },
    #[clap(
        about = "Watches for changes in the project and builds and executes love automatically."
    )]
    Dev { path: Option<PathBuf> },

    #[clap(about = "Updates the polyfill used")]
    UpdatePolyfill,
}

#[derive(Parser, Debug)]
#[clap(version)]
pub struct CLI {
    #[command(subcommand)]
    pub cli: Commands,
    // TODO: make subcommands shortcuts to flags
}

pub async fn handle_commands(command: Commands) {
    match command {
        Commands::Init { path } => {
            init::init(path);
        }
        Commands::Build { path, one_file } => {
            build::build(path, build::Strategy::Build, one_file)
                .await
                .unwrap();
        }
        Commands::Dev { path } => {
            watch::watch(path).await;
        }
        Commands::Compile { path, one_file } => {
            build::build(path, build::Strategy::BuildAndCompile, one_file)
                .await
                .unwrap();
        }
        Commands::UpdatePolyfill => {
            update_polyfill::update_polyfill().await.unwrap();
        }
    }
}
