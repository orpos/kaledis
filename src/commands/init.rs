use std::{env, fs, io::Write, path::PathBuf};

use colored::Colorize;
use inquire::{Confirm, MultiSelect, Select, Text};
use strum::IntoEnumIterator;

use crate::{
    toml_conf::{self, Config, Modules, Project, Window},
    utils::relative,
};

pub fn replace_bytes<T>(source: &mut Vec<T>, from: &[T], to: &[T])
where
    T: Clone + PartialEq,
{
    let mut result = source;
    let from_len = from.len();
    let to_len = to.len();

    let mut i = 0;
    while i + from_len <= result.len() {
        if result[i..].starts_with(from) {
            result.splice(i..i + from_len, to.iter().cloned());
            i += to_len;
        } else {
            i += 1;
        }
    }
}

pub fn init(path: Option<PathBuf>) {
    let local = relative(path);

    println!(
        "{} {}{}",
        "Initializing project in ".blue(),
        local.as_os_str().to_string_lossy().bright_white(),
        ".".blue()
    );

    let project_name = Text::new("What is the name of the project? ")
        .with_placeholder("my-game")
        .prompt()
        .unwrap();

    // TODO: give user the option to auto install
    let mut path_ = env::var_os("PATH")
        .map(|path_| {
            for path in env::split_paths(&path_) {
                if path.join("love.exe").exists() {
                    return Some(path);
                }
            }
            return None;
        })
        .flatten();
    path_ = path_.map(|x| Some(x)).unwrap_or_else(|| {
        if cfg!(windows) {
            if let Ok(dir) = env::var("ProgramFiles") {
                let love_path = PathBuf::from(dir).join("LOVE");
                if love_path.exists() {
                    return Some(love_path);
                }
            }
        }
        return None;
    });

    let location = path_.unwrap_or_else(|| {
        println!("{} {}", "[!]".red(), "Love not found.");
        Text::new("Where is the Love2D executable located?")
            .with_placeholder(r"C:\Program Files\LOVE")
            .prompt()
            .unwrap()
            .into()
    });

    let type_of_modules = Select::new(
        "Select which type of module detection you want:",
        vec!["manual", "automatic"],
    )
    .prompt()
    .unwrap();

    let modules: Vec<Modules>;
    if type_of_modules == "manual" {
        modules = MultiSelect::new(
            "Select what modules you will use:",
            Modules::iter().collect(),
        )
        .with_all_selected_by_default()
        .prompt()
        .unwrap();
    } else {
        modules = Modules::iter().collect();
    }

    let use_pesde = Confirm::new("Do you want to use pesde packages?")
        .with_default(true)
        .prompt()
        .unwrap();
    let use_src_folder = Confirm::new("Do you want to use a src folder?")
        .with_default(true)
        .prompt()
        .unwrap();
    let use_assets_folder = Confirm::new("Do you want to use a assets folder?")
        .with_default(true)
        .prompt()
        .unwrap();

    let config = toml_conf::Config {
        modules,
        project: Project {
            name: project_name.clone(),
            love_path: location.clone(),
            src_path: if use_src_folder {
                Some("src".to_string())
            } else {
                None
            },
            asset_path: if use_assets_folder {
                Some("assets".to_string())
            } else {
                None
            },
            detect_modules: if type_of_modules == "manual" {
                None
            } else {
                Some(true)
            },
            ..Default::default()
        },
        ..Default::default()
    };
    let conf = "\"$schema\" = \"./kaledis.schema.json\"\n".to_string()
        + &toml::to_string(&config).unwrap();

    if !local.exists() {
        fs::create_dir(&local).unwrap();
    }

    macro_rules! create {
        (dir $nome:expr) => {
            fs::create_dir(local.join($nome)).unwrap()
        };
        (dir $nome:expr, $($nome_2:expr),+) => {
            create!(dir $nome);
            create!(dir $($nome_2), +);
        };
        (file $nome:expr, $content:expr) => {
            {
                let mut file = fs::File::create(local.join($nome)).unwrap();
                file.write($content).unwrap();
                file
            }
        };
        (file_absolute $nome:expr, $content:expr) => {
            {
                let mut file = fs::File::create($nome).unwrap();
                file.write($content).unwrap();
                file
            }
        };
    }
    let schema = schemars::schema_for!(Config);
    create!(dir ".vscode");
    create!(file "kaledis.schema.json", serde_json::to_string_pretty(&schema).unwrap().as_bytes());
    if use_assets_folder {
        create!(dir "assets");
    }
    if use_pesde {
        create!(dir "luau_packages");
        let mut pesde_package = include_bytes!("../../static/pesde.toml").to_vec();
        replace_bytes(
            &mut pesde_package,
            b"__package_name",
            &project_name.as_bytes(),
        );
        create!(file "pesde.toml", pesde_package.as_slice());
        create!(file ".luaurc", include_bytes!("../../static/.luaurc"));
    }
    if use_src_folder {
        create!(dir "src");
        create!(file "src/main.luau", include_bytes!("../../static/main.luau"));
    } else {
        create!(file "main.luau", include_bytes!("../../static/main.luau"));
    }
    create!(file "kaledis.toml", conf.as_bytes());
    if let None = std::env::home_dir() {
        create!(file "globals.d.luau", include_bytes!("../../static/globals.d.luau"));
        create!(file_absolute local.join(".vscode").join("settings.json"), include_bytes!("../../static/vscode_settings_local.json"));
    } else {
        create!(file_absolute local.join(".vscode").join("settings.json"), include_bytes!("../../static/vscode_settings.json"));
    }
    create!(file ".gitignore", include_bytes!("../../static/.gitignore"));
}
