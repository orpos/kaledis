mod init;
mod build;
mod watch;
mod update;

use std::{ env::current_exe, path::PathBuf, thread, time::Duration };

use clap::{ Parser, Subcommand };
use tokio::{ fs::{ copy, remove_file }, process::Command };

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[clap(about = "Initializes a new Love2D project.")] Init {
        path: Option<PathBuf>,
    },
    #[clap(
        about = "Transpiles everything, and builds a '.love' file inside a '.build' directory."
    )] Build {
        path: Option<PathBuf>,
        #[arg(short, long, help="A config that joins all files in a single one.")]
        one_file: bool
    },
    #[clap(
        about = "Compiles the entire project to a executable, inside a 'dist' folder."
    )] Compile {
        path: Option<PathBuf>,
        #[arg(short, long, help="A config that joins all files in a single one.")]
        one_file: bool
    },
    #[clap(
        about = "Watches for changes in the project and builds and executes love automatically."
    )] Dev {
        path: Option<PathBuf>,
    },
    #[clap(
        about = "Updates kaledis based of github releases. This will also be used internally to handle files. If you want just to update just call it without passing anything"
    )] Update {
        step: Option<PathBuf>,
        is_established: Option<String>,
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
        Commands::Build { path, one_file } => {
            build::build(path, build::Strategy::Build, one_file).await.unwrap();
        }
        Commands::Dev { path } => {
            watch::watch(path).await;
        }
        Commands::Compile { path, one_file } => {
            build::build(path, build::Strategy::BuildAndCompile, one_file).await.unwrap();
        }
        Commands::Update { step, is_established } => {
            // Artificial delay to wait for the previous instance to die
            if let Some(_) = is_established {
                println!("Removing temporary file");
                thread::sleep(Duration::from_millis(700));
                remove_file(step.unwrap()).await.unwrap();
                return;
            }
            if let Some(target) = step {
                println!("Removing old version");
                thread::sleep(Duration::from_millis(700));
                remove_file(&target).await.unwrap();
                copy(current_exe().unwrap(), &target).await.unwrap();
                Command::new(target)
                    .args(vec!["update", &current_exe().unwrap().display().to_string(), "true"])
                    .spawn()
                    .unwrap();
                return;
            }
            update::update().await;
        }
    }
}
