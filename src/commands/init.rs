use std::{env, fs, io::Write, path::PathBuf};

use colored::Colorize;
use inquire::{Confirm, MultiSelect, Select, Text};
use strum::IntoEnumIterator;

use crate::{
    toml_conf::{KaledisConfig, Modules},
    utils::relative,
};

pub fn replace_bytes<T>(source: &mut Vec<T>, from: &[T], to: &[T])
where
    T: Clone + PartialEq,
{
    let result = source;
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

    let love = Select::new(
        "Select which version of love you want:",
        vec!["11.5", "11.4", "11.3", "11.2", "11.1", "11.0"],
    )
    .prompt()
    .unwrap();

    let type_of_modules = Select::new(
        "Select which type of module detection you want:",
        vec!["automatic", "manual"],
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

    let conf = format!(
        r#""$schema" = "https://raw.githubusercontent.com/orpos/kaledis/refs/heads/main/static/kaledis.schema.json"

# Used for macos and android apps
game_id = "com.game.{}"
project_name = "{}"
love = "{}"
{}{}{}{}
"#,
        &project_name,
        &project_name,
        &love,
        if type_of_modules == "manual" {
            format!(
                "modules=[{}]\n",
                modules
                    .iter()
                    .map(|x| format!("\"{}\"", &x))
                    .collect::<Vec<String>>()
                    .join(",")
            )
        } else {
            "detect_modules = true\n".to_string()
        },
        if use_src_folder || use_assets_folder {
            "\n[layout]\n"
        } else {
            ""
        },
        if use_src_folder {
            "code = \"src\"\n"
        } else {
            ""
        },
        if use_assets_folder {
            r#"bundle = ["assets/bundle/*"]
external = ["assets/external/*"]
"#
        } else {
            ""
        }
    );

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
    // let schema = schemars::schema_for!(Config);
    create!(dir ".vscode");
    // create!(file "kaledis.schema.json", serde_json::to_string_pretty(&schema).unwrap().as_bytes());
    if use_assets_folder {
        create!(dir "assets");
        create!(dir "assets/external");
        create!(dir "assets/bundle");
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

    create!(file "assets/external/readme.txt", b"This folder be included outside of the .love bundle");
    create!(file "assets/bundle/readme.txt", b"This folder be included inside the .love bundle");
    create!(file "kaledis.toml", conf.as_bytes());
    create!(file "conf.toml", r#"
"$schema" = "https://raw.githubusercontent.com/orpos/kaledis/refs/heads/main/static/love.schema.json"

# if you want to use a custom luau conf, just remove this file and put an conf.luau in the root folder

[audio]

[project]
name = "Test"

[window]
"#.as_bytes());
    if let None = std::env::home_dir() {
        create!(file "globals.d.luau", include_bytes!("../../static/globals.d.luau"));
        create!(file_absolute local.join(".vscode").join("settings.json"), include_bytes!("../../static/vscode_settings_local.json"));
    } else {
        create!(file_absolute local.join(".vscode").join("settings.json"), include_bytes!("../../static/vscode_settings.json"));
    }
    create!(file ".gitignore", include_bytes!("../../static/.gitignore"));
}
