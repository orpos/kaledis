mod init;
mod build;
mod watch;

use std::path::PathBuf;

use clap::{ Parser, Subcommand };

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[clap(about = "Initializes a new Love2D project")] Init {
        path: Option<PathBuf>,
    },
    #[clap(about = "Transpiles everything, and builds a '.love' file.")] Build {
        path: Option<PathBuf>,
    },
    #[clap(about = "Compiles the entire project to a executable")] Compile {
        path: Option<PathBuf>,
    },
    #[clap(about = "Runs the love 2d project.")] Dev {
        path: Option<PathBuf>,
    },
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
        Commands::Build { path } => {
            build::build(path, build::Strategy::Build).await.unwrap();
        }
        Commands::Dev { path } => {
            watch::watch_folder(path).await;
        }
        Commands::Compile { path } => {
            build::build(path, build::Strategy::BuildAndCompile).await.unwrap();
        }
    }
}
