// pub mod build;
pub mod init;
pub mod update_polyfill;
// pub mod watch;
// mod update;
pub mod android;
pub mod build;
pub mod build_utils;
pub mod watch;

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use tokio::fs;

use crate::{commands::init::replace_bytes, toml_conf::Config};

#[derive(ValueEnum, Clone, Debug)]
pub enum Features {
    Assets,
    Pesde,
    Zed,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[clap(about = "Initializes a new Love2D project.")]
    Init { path: Option<PathBuf> },
    #[clap(about = "Setups a feature in your project")]
    Setup { feature: Features },
    #[clap(about = "Transpiles everything, and builds a '.love' file inside a '.build' directory.")]
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

    #[clap(
        about = "Watches for changes in the project and builds and executes love automatically."
    )]
    AndroidDev { ip: String, path: Option<PathBuf> },

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
    // automatically adds the globals
    if let Some(user_dir) = std::env::home_dir() {
        let err = fs::create_dir(user_dir.join(".kaledis")).await;
        let f_err = fs::write(
            user_dir.join(".kaledis").join("globals.d.luau"),
            include_bytes!("../../static/globals.d.luau"),
        )
        .await;

        // Shows errors
        if err.is_err() {
            // TODO: ignore already created dir error
            // eprintln!("{:?}", err.unwrap_err());
        }
        if f_err.is_err() {
            eprintln!("{:?}", f_err.unwrap_err());
        }
    }

    match command {
        Commands::AndroidDev { path, ip } => {
            android::watch(path, ip).await;
        }
        Commands::Init { path } => {
            init::init(path);
        }
        Commands::Setup { feature } => {
            let config = Config::from_toml_file("kaledis.toml").expect("Project not found!");
            macro_rules! create {
                (dir $nome:expr) => {
                    if !fs_err::tokio::try_exists($nome).await.unwrap_or(false) {
                        fs_err::tokio::create_dir($nome).await.unwrap()
                    }
                };
                (dir $nome:expr, $($nome_2:expr),+) => {
                    create!(dir $nome);
                    create!(dir $($nome_2), +);
                };
                (file $nome:expr, $content:expr) => {
                    fs_err::tokio::write($nome, $content).await.unwrap()
                };
            }
            match &feature {
                Features::Assets => {
                    if config.project.asset_path.is_some() {
                        println!(
                            "{} Assets are already configured, try changing your kaledis.toml to your new assets folder",
                            "[~]".yellow()
                        );
                        return;
                    }
                    create!(dir "assets");
                    let contents = fs_err::tokio::read_to_string("kaledis.toml").await.unwrap();
                    let new_contents =
                        contents.replace("[project]", "[project]\nasset_path=\"assets\"");
                    create!(file "kaledis.toml", new_contents);
                    println!("Setup successful");
                }
                Features::Pesde => {
                    create!(dir "luau_packages");
                    let mut pesde_package = include_bytes!("../../static/pesde.toml").to_vec();
                    replace_bytes(
                        &mut pesde_package,
                        b"__package_name",
                        &config.project.name.as_bytes(),
                    );
                    create!(file "pesde.toml", pesde_package.as_slice());
                    create!(file ".luaurc", include_bytes!("../../static/.luaurc"));
                    println!("Setup successful");
                }
                Features::Zed => {
                    create!(dir ".zed");

                    create!(file ".zed/settings.json", include_bytes!("../../static/zed_settings.json"));
                    create!(file "globals.d.luau", include_bytes!("../../static/globals.d.luau"));

                    println!("Setup successful");
                }
            }
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
