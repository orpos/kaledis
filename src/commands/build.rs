use std::str::FromStr;
use std::path::PathBuf;
use std::sync::{ Arc, Mutex };

use anyhow::Context;
use indexmap::IndexSet;
use strum::IntoEnumIterator;
use tokio::io::{ AsyncReadExt, AsyncWriteExt };
use tokio::fs::{ self, create_dir, remove_dir_all, File };

use kaledis_dalbit::{ manifest::Manifest, transpile };

use crate::cli_utils::LoadingStatusBar;
use crate::{ toml_conf::{ Config, Modules }, utils::relative };
use colored::Colorize;
use crate::{ allow, zip_utils::* };

pub enum ModelConf {
    Exclude(Vec<Modules>),
    Include(Vec<Modules>),
}

impl From<&Config> for ModelConf {
    fn from(config: &Config) -> Self {
        if config.modules.len() < 1 && config.exclude_modules.len() > 0 {
            Self::Exclude(config.exclude_modules.clone())
        } else if config.modules.len() > 0 {
            if config.exclude_modules.len() > 0 {
                eprintln!(
                    "{}",
                    "Both modules and exclude modules used, the exclude modules will be ignored".red()
                );
            }
            Self::Include(config.modules.clone())
        } else {
            Self::Exclude(vec![])
        }
    }
}

struct Builder {
    transpiler_manifest: Manifest,
    strategy: Strategy,
    zip: Option<Zipper>,
    local: PathBuf,
    build_path: PathBuf,
    bar: LoadingStatusBar,
    used_modules: Option<Arc<Mutex<IndexSet<String>>>>,
}

impl Builder {
    pub async fn new(
        local: &PathBuf,
        collect_used_modules: bool,
        run: Strategy,
        one_file: bool
    ) -> anyhow::Result<Self> {
        let config = get_transpiler(one_file).await.context("Failed to build manifest")?;
        let bar = LoadingStatusBar::new("Building project...".into());
        bar.start_animation().await;
        Ok(Self {
            transpiler_manifest: config,
            zip: if let Strategy::BuildDev = run {
                None
            } else {
                Some(Zipper::new())
            },
            strategy: run,
            bar,
            build_path: local.join(".build"),
            local: local.clone(),
            used_modules: if collect_used_modules {
                Some(Arc::new(Mutex::new(IndexSet::new())))
            } else {
                None
            },
        })
    }

    pub fn generate_conf_modules(
        &self,
        imported_modules: Vec<Modules>,
        model_conf: ModelConf
    ) -> String {
        let mut modules_string = "".to_string();
        for module in imported_modules {
            let enabled = match model_conf {
                ModelConf::Include(ref models) | ModelConf::Exclude(ref models) => {
                    let found_model = models
                        .iter()
                        .find(|x| **x == module)
                        .is_some();
                    if let ModelConf::Exclude(_) = model_conf {
                        !found_model
                    } else {
                        found_model
                    }
                }
            };
            modules_string += &format!(
                "t.modules.{}={}\n",
                &module.to_string().to_lowercase(),
                enabled
            );
        }
        modules_string
    }
    pub async fn clean_build_folder(&self) -> anyhow::Result<()> {
        if self.build_path.exists() {
            // This clean function only happens when a new build is requested
            // and in dev i considered it unnecessary to persist
            if let Strategy::BuildDev = self.strategy {
                self.bar.change_status("Cleaning build folder.".to_string()).await;
            } else {
                println!("Previous build folder found. Deleting it...");
            }
            remove_dir_all(&self.build_path).await?;
        }
        create_dir(&self.build_path).await?;
        Ok(())
    }
    pub async fn process_file(&self, input: PathBuf, output: PathBuf) -> anyhow::Result<()> {
        let mut additional_rules = vec![
            kaledis_dalbit::modifiers::Modifier::DarkluaRule(
                Box::new(kaledis_dalbit::modifiers::ModifyRelativePath {
                    project_root: self.local.clone(),
                })
            )
        ];
        if let Some(modules) = &self.used_modules {
            additional_rules.push(
                kaledis_dalbit::modifiers::Modifier::DarkluaRule(
                    Box::new(kaledis_dalbit::modifiers::GetLoveModules {
                        modules: Arc::clone(modules),
                    })
                )
            );
        }
        let mut new_manifest = self.transpiler_manifest.clone();
        new_manifest.input = input;
        new_manifest.output = output;
        new_manifest.minify = if self.strategy == Strategy::BuildDev { true } else { false };
        transpile::process(new_manifest, Some(&mut additional_rules)).await?;
        return Ok(());
    }
    pub async fn add_luau_file(&mut self, input: &PathBuf) -> anyhow::Result<()> {
        let zip_path = input.strip_prefix(&self.local)?;

        let out_path = self.local.join(".build").join(zip_path);
        self.process_file(input.clone(), out_path).await?;

        if let Some(zip) = &mut self.zip {
            zip.copy_zip_f_from_path(
                &self.local.join(".build").join(zip_path).with_extension("lua"),
                zip_path.with_extension("lua")
            ).await?;
        }

        Ok(())
    }
    pub async fn add_luau_files(&mut self) -> anyhow::Result<Vec<Modules>> {
        self.bar.change_status(format!("{} {} {}", "Adding", "lua".green(), "files...")).await;
        if self.local.join("main.luau").exists() && self.transpiler_manifest.bundle {
            if let Err(dat) = self.add_luau_file(&self.local.join("main.luau")).await {
                eprintln!("{:?}", dat);
                panic!("{} Failed to process {} file", "[!]".red(), "main.luau");
            }
        } else {
            for path in glob
                ::glob(&(self.local.to_string_lossy().to_string() + "/**/*.luau"))?
                .filter_map(Result::ok)
                .filter(
                    |path|
                        !path
                            .file_name()
                            .map(|x| x.to_string_lossy().to_string())
                            .unwrap_or("".to_string())
                            .ends_with(".d.luau")
                ) {
                if let Err(dat) = self.add_luau_file(&path).await {
                    eprintln!("{:?}", dat);
                    eprintln!("{} Failed to process {} file", "[!]".red(), path.display());
                }
            }
        }
        if let Some(zip) = &mut self.zip {
            for path in glob
                ::glob(&(self.local.to_string_lossy().to_string() + "/**/__polyfill__.lua"))
                .unwrap()
                .filter_map(Result::ok) {
                if
                    path
                        .file_name()
                        .map(|x| x.to_string_lossy().to_string())
                        .unwrap_or("".to_string())
                        .ends_with(".d.luau")
                {
                    continue;
                }
                let out_path = path.strip_prefix(&self.local.join(".build")).unwrap();
                zip.copy_zip_f_from_path(
                    &self.local.join(".build").join(out_path).with_extension("lua"),
                    out_path.with_extension("lua")
                ).await.unwrap();
            }
        }
        if let Some(modules) = &self.used_modules {
            let inside = modules.lock().unwrap();
            Ok(
                inside
                    .iter()
                    .map(|x| Modules::from_str(&uppercase_first(&x)))
                    .filter_map(Result::ok)
                    .collect()
            )
        } else {
            Ok(vec![])
        }
    }
    pub async fn add_assets(&mut self) {
        self.bar.change_status("Adding asset files...".into()).await;
        for path in glob
            ::glob(&self.local.join("**/*").display().to_string())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|pth| {
                let ext = pth
                    .extension()
                    .map(|x| x.to_str().unwrap())
                    .unwrap_or("");
                !(
                    pth.starts_with(self.local.join("dist")) ||
                    allow!(ext, "lua", "luau", "toml") ||
                    pth.is_dir()
                )
            }) {
            if let Some(zip) = &mut self.zip {
                zip.add_zip_f_from_path(&path, &self.local).await.unwrap();
            } else {
                let final_ = self.local
                    .join(".build")
                    .join(path.strip_prefix(self.local.clone()).unwrap());
                if !final_.parent().unwrap().exists() {
                    fs::create_dir_all(final_.parent().unwrap()).await.unwrap();
                }
                fs::hard_link(&path, final_).await.unwrap();
            }
        }
    }
    #[inline]
    pub fn finish_zip(mut self) -> Option<Vec<u8>> {
        self.zip.take().map(|x| x.finish())
    }
}

pub async fn get_transpiler(one_file: bool) -> anyhow::Result<Manifest> {
    let mut manifest = Manifest {
        minify: true,
        file_extension: Some("lua".to_string()),
        target_version: kaledis_dalbit::TargetVersion::Lua51,
        bundle: one_file,
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
    // Thanks to new dalbit version this was made much easier
    manifest.polyfill.cache().await?;
    return Ok(manifest);
}

fn uppercase_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

fn format_option<T: ToString>(value: Option<T>) -> String {
    value.map(|x| x.to_string()).unwrap_or("nil".to_string())
}

#[derive(PartialEq, Eq, Clone)]
pub enum Strategy {
    /// Makes the executable
    BuildAndCompile,
    /// Just creates the love file
    Build,
    /// Just compiles the lua files
    BuildDev,
}

pub async fn build(path: Option<PathBuf>, run: Strategy, one_file: bool) -> anyhow::Result<()> {
    let local = relative(path);

    if !local.join("kaledis.toml").exists() {
        println!("{}", "No Project found!".red());
        return Ok(());
    }
    let configs = Config::from_toml_file(local.join("kaledis.toml"))?;

    if configs.project.name.len() < 1 {
        eprintln!("{}", "Cannot distribute a game without a name".red());
        return Ok(());
    }

    let mut builder = Builder::new(
        &local,
        configs.project.detect_modules.unwrap_or(false),
        run.clone(),
        one_file
    ).await?;
    builder.clean_build_folder().await?;
    let imported_modules = builder.add_luau_files().await?;

    builder.add_assets().await;

    let model_conf: ModelConf = (&configs).into();

    builder.bar.change_status("Adding config file...".into()).await;
    if !(local.join("conf.luau").exists() || local.join("conf.lua").exists()) {
        let modules = builder.generate_conf_modules(
            if let Strategy::BuildDev = run {
                Modules::iter().collect()
            } else {
                if configs.project.detect_modules.unwrap_or(false) {
                    println!("Detected Modules: {:?}", imported_modules);
                    imported_modules
                } else {
                    Modules::iter().collect()
                }
            },
            model_conf
        );
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
            format_option(
                configs.project.identity.as_ref().map(|x| x.to_string_lossy().to_string())
            ),
            "false",
            configs.project.version,
            configs.project.console,
            configs.project.accelerometer_joystick,
            configs.project.external_storage,
            configs.project.gamma_correct,
            configs.audio.mic,
            configs.audio.mix_with_system,
            configs.window.title,
            format_option(configs.window.icon.as_ref().map(|x| x.to_string_lossy().to_string())),
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
            modules
        );
        if let Strategy::BuildDev = run {
            let mut result = fs::File::create(builder.build_path.join("conf.lua")).await?;
            result.write(conf_file.as_bytes()).await?;
        } else {
            if let Some(zip) = &mut builder.zip {
                zip.add_zip_f_from_buf("conf.lua", conf_file.as_bytes()).await?;
            }
        }
    } else {
        println!("{}", "Custom config file found! Overwriting configs...".yellow());
    }

    match run {
        Strategy::BuildDev => {}
        Strategy::BuildAndCompile => {
            let build_path = builder.build_path.clone();
            builder.clean_build_folder().await?;
            let fin = builder.finish_zip().unwrap();
            let love_executable = configs.project.love_path.join("love.exe");

            let mut contents = File::open(love_executable).await?;
            let mut buffer = Vec::new();

            contents.read_to_end(&mut buffer).await?;

            let dist_folder = local.join("dist");

            if !dist_folder.exists() {
                create_dir(&dist_folder).await?;
            }

            let new_exe = dist_folder.join(configs.project.name).with_extension("exe");

            let mut f = File::create(&new_exe).await?;
            f.write(&buffer).await?;
            f.write(&fin).await?;

            println!("Saving executable in : {}", new_exe.display().to_string());

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
        Strategy::Build => {
            let build_path = builder.build_path.clone();
            let fin = builder.finish_zip().unwrap();

            let mut file = File::create(build_path.join("final.love")).await?;
            file.write(&fin).await?;
        }
    }

    println!("{} {}", "[+]".green(), "Love project builded sucessfully");

    Ok(())
}
