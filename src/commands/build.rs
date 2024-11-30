use std::str::FromStr;
use std::path::PathBuf;

use tokio::io::{ AsyncReadExt, AsyncWriteExt };
use tokio::fs::{ create_dir, remove_dir_all, File };

use dal_core::{ manifest::Manifest, polyfill::Polyfill, transpiler::Transpiler };
use url::Url;

use crate::cli_utils::LoadingStatusBar;
use crate::{ toml_conf::{ Config, Modules }, utils::relative };
use colored::Colorize;
use crate::{ disallow, zip_utils::* };

pub const DEFAULT_POLYFILL_URL: &str = "https://github.com/orpos/dal-polyfill";

pub async fn get_transpiler() -> (Transpiler, Manifest) {
    let mut manifest = Manifest {
        minify: true,
        file_extension: Some("lua".to_string()),
        target_version: dal_core::TargetVersion::Lua51,
        ..Default::default()
    };

    macro_rules! add_modifiers {
        ($modifier:expr) => {
            manifest.modifiers.insert($modifier.to_string(), true);
        };
        ($modifier:expr, $($modi:expr),+) => {
            add_modifiers!($modifier);
            add_modifiers!($($modi), +);
        };
    }
    add_modifiers!(
        // "rename_variables",
        "remove_empty_do",
        "remove_spaces",
        "remove_unused_while",
        "remove_unused_variable",
        "remove_unused_if_branch"
    );

    let polyfill = Polyfill::new(&Url::from_str(DEFAULT_POLYFILL_URL).unwrap()).await.unwrap();
    polyfill.fetch().unwrap();

    let mut transpiler = Transpiler::default();
    transpiler = transpiler.with_manifest(&manifest);
    transpiler = transpiler.with_polyfill(polyfill);

    return (transpiler, manifest);
}

pub async fn process_file(
    config: &(Transpiler, Manifest),
    input: PathBuf,
    output: PathBuf
) -> anyhow::Result<()> {
    let (transpiler, manifest) = config;
    transpiler.process(
        manifest.require_input(Some(input)).unwrap(),
        manifest.require_output(Some(output)).unwrap()
    ).await?;
    Ok(())
}

pub async fn add_luau_files(
    config: &(Transpiler, Manifest),
    local: &PathBuf,
    zip: &mut Zipper
) -> anyhow::Result<()> {
    for entry in glob::glob(&(local.to_string_lossy().to_string() + "/**/*.luau")).unwrap() {
        if let Ok(path) = entry {
            if
                path
                    .file_name()
                    .map(|x| x.to_string_lossy().to_string())
                    .unwrap_or("".to_string())
                    .ends_with(".d.luau")
            {
                continue;
            }
            let out_path = path.strip_prefix(&local).unwrap();
            process_file(config, path.clone(), local.join(".build").join(out_path)).await?;
            zip.copy_zip_f_from_path(
                &local.join(".build").join(out_path).with_extension("lua"),
                out_path.with_extension("lua")
            ).await.unwrap();
        }
    }
    for entry in glob
        ::glob(&(local.to_string_lossy().to_string() + "/**/__dal_libs__.lua"))
        .unwrap() {
        if let Ok(path) = entry {
            if
                path
                    .file_name()
                    .map(|x| x.to_string_lossy().to_string())
                    .unwrap_or("".to_string())
                    .ends_with(".d.luau")
            {
                continue;
            }
            let out_path = path.strip_prefix(&local.join(".build")).unwrap();
            zip.copy_zip_f_from_path(
                &local.join(".build").join(out_path).with_extension("lua"),
                out_path.with_extension("lua")
            ).await.unwrap();
        }
    }
    Ok(())
}

fn format_option<T: ToString>(value: Option<T>) -> String {
    value.map(|x| x.to_string()).unwrap_or("nil".to_string())
}

pub async fn add_assets(local: &PathBuf, zip: &mut Zipper) {
    for data in glob
        ::glob(&format!("{}{}", local.to_string_lossy(), "/**/*"))
        .unwrap()
        .filter_map(Result::ok) {
        let ext = data
            .extension()
            .map(|x| x.to_str().unwrap())
            .unwrap_or("");
        if
            data.starts_with(local.join("dist")) ||
            disallow!(ext, "lua", "luau", "toml") ||
            data.is_dir()
        {
            continue;
        }
        zip.add_zip_f_from_path(&data, local).await.unwrap();
    }
}

#[derive(PartialEq, Eq)]
pub enum Strategy {
    /// Makes the executable
    BuildAndCompile,
    /// Just creates the love file
    Build,
}

pub async fn build(path: Option<PathBuf>, run: Strategy) -> anyhow::Result<()> {
    let local = relative(path);

    if !local.join("kaledis.toml").exists() {
        println!("{}", "No Project found!".red());
        return Ok(());
    }

    let configs = Config::from_toml_file(local.join("kaledis.toml")).unwrap();

    let build_path = local.join(".build");

    // Steam be like:
    if build_path.exists() {
        println!("Previous build folder found. Deleting it...");
        remove_dir_all(&build_path).await?;
    }
    create_dir(&build_path).await?;

    let transp = get_transpiler().await;

    let mut zip = Zipper::new();

    let bar = LoadingStatusBar::new("Building project...".into());
    bar.change_status(format!("{} {} {}", "Adding", "lua".green(), "files...")).await;
    bar.start_animation().await;

    add_luau_files(&transp, &local, &mut zip).await?;

    bar.change_status("Adding asset files...".into()).await;

    add_assets(&local, &mut zip).await;

    let models = {
        let mut included = Vec::new();
        if configs.modules.len() < 1 && configs.exclude_modules.len() > 0 {
            included = configs.exclude_modules;
        } else if configs.modules.len() > 0 {
            if configs.exclude_modules.len() > 0 {
                println!(
                    "{}",
                    "Both modules and exclude modules used, the exclude modules will be ignored".red()
                );
            }
            included = configs.modules;
        }
        included
    };
    let mut modules_string = "".to_string();
    for module in Modules::available() {
        modules_string += "t.modules.";
        modules_string += &module.to_string().to_lowercase();
        modules_string += "=";
        modules_string += &format!(
            "{}",
            models
                .iter()
                .find(|x| **x == module)
                .is_some()
        );
        modules_string += "\n";
    }
    if !(local.join("conf.luau").exists() || local.join("conf.lua").exists()) {
        // TODO: make this from serialize
        let conf_file = format!(
            r#"
    function love.conf(t)
        t.identity = {}
        t.appendidentity = {}
        t.version = {:?}
        t.console = {}
        t.accelerometerjoystick = {}
        t.externalstorage = {}
        t.gammacorrect = {}
    
        t.audio.mic = {}
        t.audio.mixwithsystem = {}   
    
        t.window.title = {:?}
        t.window.icon = {}
        t.window.width = {}
        t.window.height = {}
        t.window.borderless = {}
        t.window.resizable = {}
        t.window.minwidth = {}
        t.window.minheight = {}
        t.window.fullscreen = {}
        t.window.fullscreentype = {}
        t.window.vsync = {}
        t.window.msaa = {}
        t.window.depth = {}
        t.window.stencil = {}
        t.window.display = {}
        t.window.highdpi = {}
        t.window.usedpiscale = {}
        t.window.x = {}
        t.window.y = {}
        {}
    end
        "#,
            format_option(configs.project.identity.map(|x| x.to_string_lossy().to_string())),
            "false",
            configs.project.version,
            configs.project.console,
            configs.project.accelerometer_joystick,
            configs.project.external_storage,
            configs.project.gamma_correct,
            configs.audio.mic,
            configs.audio.mix_with_system,
            configs.window.title,
            format_option(configs.window.icon.map(|x| x.to_string_lossy().to_string())),
            configs.window.width,
            configs.window.height,
            configs.window.borderless,
            configs.window.resizable,
            configs.window.minwidth,
            configs.window.minheight,
            configs.window.fullscreen,
            match configs.window.fullscreentype {
                crate::toml_conf::FullscreenType::Desktop => "\"desktop\"",
                crate::toml_conf::FullscreenType::Exclusive => "\"exclusive\"",
            },
            configs.window.vsync,
            configs.window.msaa,
            format_option(configs.window.depth),
            format_option(configs.window.stencil),
            configs.window.display,
            configs.window.highdpi,
            configs.window.usedpiscale,
            format_option(configs.window.x),
            format_option(configs.window.y),
            modules_string
        );
        zip.add_zip_f_from_buf("conf.lua", conf_file.as_bytes()).await?;
    } else {
        println!("{}", "Custom config file found! Overwriting configs...".yellow());
    }

    bar.change_status("Adding config file...".into()).await;
    // println!("{} {}", "[-]".blue(), "Adding config file...");

    let fin = zip.finish();
    match run {
        Strategy::Build => {
            let mut file = File::create(build_path.join("final.love")).await?;
            file.write(&fin).await?;
        }
        Strategy::BuildAndCompile => {
            let love_executable = configs.project.love_path.join("love.exe");

            let mut contents = File::open(love_executable).await?;
            let mut buffer = Vec::new();

            contents.read_to_end(&mut buffer).await?;

            let dist_folder = local.join("dist");

            if !dist_folder.exists() {
                create_dir(&dist_folder).await?;
            }

            let mut f = File::create(
                dist_folder.join(configs.project.name).with_extension("exe")
            ).await?;
            f.write(&buffer).await?;
            f.write(&fin).await?;

            let l_path = configs.project.love_path;

            macro_rules! import_love_file {
                ($name:expr) => {
                    {
                        let path = l_path.join($name);
                        if path.exists() {
                            std::fs::copy(&path, dist_folder.join($name))?;
                        } else {
                            println!("{}{:?}", "Missing dll: ".red(), path);
                        }
                    }
                };
                ($name:expr, $($na:expr),+) => {
                    import_love_file!($name);
                    import_love_file!($($na), +)
                };
            }
            import_love_file!(
                "license.txt",
                "love.dll",
                "lua51.dll",
                "mpg123.dll",
                "msvcp120.dll",
                "msvcr120.dll",
                "OpenAL32.dll",
                "SDL2.dll"
            );
            remove_dir_all(&build_path).await?;
        }
    }

    println!("{} {}", "[+]".green(), "Love project builded sucessfully");

    return Ok(());
}
