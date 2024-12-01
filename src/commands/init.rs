use std::{ env, fs, io::Write, path::PathBuf };

use colored::Colorize;
use inquire::{ MultiSelect, Text };
use strum::IntoEnumIterator;

use crate::{ toml_conf::{ self, Modules, Project }, utils::relative };

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
    let mut path_ = env
        ::var_os("PATH")
        .map(|path_| {
            for path in env::split_paths(&path_) {
                if path.join("love.exe").exists() {
                    return Some(path);
                }
            }
            return None;
        })
        .flatten();
    path_ = path_
        .map(|x| Some(x))
        .unwrap_or_else(|| {
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

    let modules = MultiSelect::new("Select what modules you will use:", Modules::iter().collect())
        .with_all_selected_by_default()
        .prompt()
        .unwrap();

    let config = toml_conf::Config {
        modules,
        project: Project {
            name: project_name,
            love_path: location.clone(),
            ..Default::default()
        },
        ..Default::default()
    };
    let conf = toml::to_string(&config).unwrap();

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
    create!(dir "modules", "assets", ".vscode");
    create!(file "kaledis.toml", conf.as_bytes());
    create!(file "globals.d.luau", include_bytes!("../../static/globals.d.luau"));
    create!(file "main.luau", include_bytes!("../../static/main.luau"));
    create!(file_absolute local.join(".vscode").join("settings.json"), include_bytes!("../../static/vscode_settings.json"));
    create!(file ".gitignore", include_bytes!("../../static/.gitignore"));
}
