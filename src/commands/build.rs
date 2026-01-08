use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use crate::dalbit::manifest::Manifest;
use crate::dalbit::transpile::clean_polyfill;
use crate::dalbit::transpile::get_polyfill_contents;
use crate::dalbit::transpile::process_files;
use colored::Colorize;
use ignore::WalkBuilder;
use indexmap::IndexSet;
use indicatif::MultiProgress;
use indicatif::ProgressBar;
use std::sync::Mutex;
use strum::IntoEnumIterator;
use tokio::fs;
use tokio::fs::File;
use tokio::fs::create_dir;
use tokio::fs::read;
use tokio::fs::remove_dir_all;
use tokio::fs::write;
use tokio::io::AsyncWriteExt;

use super::build_utils::read_aliases;
use crate::allow;
use crate::commands::build_utils::Paths;
use crate::commands::build_utils::get_transpiler;
use crate::commands::build_utils::uppercase_first;
use crate::toml_conf::Config;
use crate::toml_conf::Modules;
use crate::utils::relative;
use crate::zip_utils::Zipper;

#[derive(PartialEq, Eq, Clone)]
pub enum Strategy {
    /// Makes the executable
    BuildAndCompile,
    /// Just creates the love file
    Build,
    /// Just compiles the lua files
    BuildDev,
}

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
                    "Both modules and exclude modules used, the exclude modules will be ignored"
                        .red()
                );
            }
            Self::Include(config.modules.clone())
        } else {
            Self::Exclude(vec![])
        }
    }
}

pub struct Builder {
    pub config: Config,
    pub strategy: Strategy,
    pub aliases: Vec<(String, String)>,
    pub used_modules: Option<Arc<Mutex<IndexSet<String>>>>,
    pub progress_bar: MultiProgress,
    pub transpiler_manifest: Manifest,
    pub paths: Paths,
}

impl Builder {
    pub async fn update_file(
        path: &PathBuf,
        name: &str,
        new_contents: &[u8],
        should_create: bool,
    ) -> anyhow::Result<()> {
        if !path.exists() && should_create {
            File::create(path)
                .await
                .unwrap()
                .write(new_contents)
                .await
                .unwrap();
            return Ok(());
        }
        if !path.exists() && !should_create {
            return Ok(());
        }
        let contents = read(path).await?;
        if contents != new_contents {
            println!("Updating {}", name);
            if let Err(e) = write(path, new_contents).await {
                eprintln!("{:?}", e);
                eprintln!("Failed to update {}, Resuming...", name);
            } else {
                println!("Updated {} schema", name);
            };
        }
        Ok(())
    }
    //
    // Main code part
    //
    pub async fn new(
        root: PathBuf,
        config: Config,
        strategy: Strategy,
        is_bundle: bool,
    ) -> anyhow::Result<Self> {
        clean_polyfill();
        let polyfill = config.polyfill.clone();
        Self::update_file(
            &root.join("globals.d.luau"),
            "globals.d.luau",
            include_bytes!("../../static/globals.d.luau"),
            false,
        )
        .await?;
        Self::update_file(
            &root.join("kaledis.schema.json"),
            "kaledis.toml schema",
            &{
                let schema = schemars::schema_for!(Config);
                let content_str = serde_json::to_string_pretty(&schema).unwrap();
                content_str.as_bytes().to_vec()
            },
            true,
        )
        .await?;
        Ok(Self {
            aliases: read_aliases(&root).await.unwrap(),
            progress_bar: MultiProgress::new(),
            strategy,
            used_modules: if config.project.detect_modules.unwrap_or(false) {
                Some(Arc::new(Mutex::new(IndexSet::new())))
            } else {
                None
            },
            paths: Paths::from_root(root, &config),
            config: config,
            transpiler_manifest: get_transpiler(is_bundle, polyfill.as_ref()).await.unwrap(),
        })
    }
    pub async fn make_conf_file(&self, modules: Vec<Modules>) {
        let model_conf: ModelConf = (&self.config).into();
        let modules = self.generate_conf_modules_lua(
            if Strategy::BuildDev == self.strategy
                || self.config.project.detect_modules.unwrap_or(false)
            {
                Modules::iter().collect()
            } else {
                println!("Detected Modules: {:?}", modules);
                modules
            },
            model_conf,
        );
        let conf_file = format!(
            r#"function love.conf(t)
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
end"#,
            format_option(
                self.config
                    .project
                    .identity
                    .as_ref()
                    .map(|x| x.to_string_lossy().to_string())
            ),
            "false",
            self.config.project.version,
            self.config.project.console,
            self.config.project.accelerometer_joystick,
            self.config.project.external_storage,
            self.config.project.gamma_correct,
            self.config.audio.mic,
            self.config.audio.mix_with_system,
            self.config.window.title,
            format_option(
                self.config
                    .window
                    .icon
                    .as_ref()
                    .map(|x| x.to_string_lossy().to_string())
            ),
            self.config.window.width,
            self.config.window.height,
            self.config.window.borderless,
            self.config.window.resizable,
            self.config.window.minwidth,
            self.config.window.minheight,
            self.config.window.fullscreen,
            match self.config.window.fullscreentype {
                crate::toml_conf::FullscreenType::Desktop => "\"desktop\"",
                crate::toml_conf::FullscreenType::Exclusive => "\"exclusive\"",
            },
            self.config.window.vsync,
            self.config.window.msaa,
            format_option(self.config.window.depth),
            format_option(self.config.window.stencil),
            self.config.window.display,
            self.config.window.highdpi,
            self.config.window.usedpiscale,
            format_option(self.config.window.x),
            format_option(self.config.window.y),
            modules
        );
        if let Strategy::BuildDev = self.strategy {
            let mut result = fs::File::create(self.paths.build.join("conf.lua"))
                .await
                .unwrap();
            result.write(conf_file.as_bytes()).await.unwrap();
        }
    }

    /// Build steps
    pub async fn clean_build_folder(&self) -> anyhow::Result<()> {
        if self.paths.build.exists() {
            // This clean function only happens when a new build is requested
            // and in dev i considered it unnecessary to persist
            let mut p = ProgressBar::new_spinner().with_message("Cleaning Build Folder...");
            p = self.progress_bar.add(p);
            remove_dir_all(&self.paths.build).await?;
            p.finish_with_message(format!("{} Build Folder cleaned", "[+]".green()));
        }
        create_dir(&self.paths.build).await?;
        Ok(())
    }
    pub fn generate_conf_modules_lua(
        &self,
        imported_modules: Vec<Modules>,
        model_conf: ModelConf,
    ) -> String {
        let mut modules_string = String::new();
        for module in imported_modules {
            let enabled = match model_conf {
                ModelConf::Include(ref models) | ModelConf::Exclude(ref models) => {
                    let found_model = models.iter().find(|x| **x == module).is_some();
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
    pub async fn add_assets(&mut self) {
        // TODO: used override builder
        if let None = self.paths.assets {
            return;
        }
        let assets = self.paths.assets.as_ref().unwrap();
        let mut p = ProgressBar::new_spinner().with_message("Adding assets...");
        p = self.progress_bar.add(p);
        for path in WalkBuilder::new(assets)
            .build()
            .filter_map(Result::ok)
            .filter(|path| {
                let pth = path.path();
                let ext = pth.extension().map(|x| x.to_str().unwrap()).unwrap_or("");
                !(pth.starts_with(self.paths.root.join("dist"))
                    || allow!(ext, "lua", "luau", "toml")
                    || pth.starts_with(self.paths.root.join("luau_packages"))
                    || pth.ends_with(self.paths.root.join("kaledis.schema.json"))
                    || pth.is_dir())
            })
        {
            let path = path.path();
            let final_ = self
                .paths
                .build
                .join(path.strip_prefix(&self.paths.root).unwrap());
            if !final_.parent().unwrap().exists() {
                fs::create_dir_all(final_.parent().unwrap()).await.unwrap();
            }
            p.inc(1);
            fs::hard_link(&path, final_).await.unwrap();
        }
        p.finish_with_message(format!("{} Added assets", "[+]".green()));
    }
    pub async fn build_files(&mut self, paths: Vec<PathBuf>) {
        let mut file_map = HashMap::new();
        for file in paths {
            let new_value = self.paths.build.join(
                file.with_extension("lua")
                    .strip_prefix(&self.paths.src)
                    .unwrap_or(
                        file.with_extension("lua")
                            .strip_prefix(&self.paths.root)
                            .unwrap(),
                    ),
            );
            file_map.insert(file, new_value);
        }
        process_files(
            &self.transpiler_manifest,
            file_map,
            self.paths.clone(),
            &self.aliases,
        )
        .unwrap();
    }
    pub async fn add_luau_files(&mut self) -> anyhow::Result<Vec<Modules>> {
        if self.config.experimental_hmr {
            let _ = tokio::fs::write(
                self.paths.build.join("lick.lua"),
                include_bytes!("../../static/lick.lua"),
            )
            .await;
        }
        // To maintain compatibility with lua better we are also putting the lua file
        for lua_file in glob::glob(
            &self
                .paths
                .src
                .join("**/*.lua")
                .to_string_lossy()
                .to_string(),
        )?
        .filter_map(Result::ok)
        {
            // here we don't copy the file, we just create a link to it
            // it's like a pointer to the original data
            let _ = tokio::fs::hard_link(
                lua_file.clone(),
                self.paths
                    .build
                    .join(&lua_file.strip_prefix(&self.paths.src).unwrap()),
            )
            .await;
        }
        let mut p = ProgressBar::new_spinner().with_message("Processing luau files");
        p = self.progress_bar.add(p);
        self.transpiler_manifest.hmr =
            self.config.experimental_hmr && self.strategy == Strategy::BuildDev;
        // Bundles everything in one path
        if self.paths.src.join("main.luau").exists() && self.transpiler_manifest.bundle {
            let mut path = self.paths.root.join("main.luau");
            if !path.exists() {
                path = self.paths.src.join("main.luau");
            }
            self.build_files(vec![path]).await;
        } else {
            let filter_glob = |x: glob::Paths| {
                let files: Vec<PathBuf> = x
                    .filter_map(Result::ok)
                    .filter(|path| {
                        let file = path
                            .file_name()
                            .map(|x| x.to_string_lossy().to_string())
                            .unwrap_or("".to_string());
                        if let Some(polyfill_path) = &self.paths.polyfill_path {
                            if path.starts_with(polyfill_path) {
                                return false;
                            }
                        }
                        !file.ends_with(".d.luau")
                    })
                    .collect();
                files
            };
            // Don't try to use iter.
            let mut luau_files: Vec<_> = filter_glob(glob::glob(
                &self.paths.src.join("**/*.luau").to_string_lossy(),
            )?);
            for (_, path) in &self.aliases {
                let abs;
                if path.starts_with(".") {
                    abs = std::fs::canonicalize(self.paths.root.join(path)).unwrap();
                } else {
                    abs = self.paths.root.join(PathBuf::from_str(&path).unwrap());
                }
                luau_files.append(&mut filter_glob(glob::glob(
                    &abs.join("**/*.luau").to_string_lossy(),
                )?));
            }
            self.build_files(luau_files).await;
        }
        let mut file = File::create(self.paths.build.join("__polyfill__.lua"))
            .await
            .unwrap();
        file.write(get_polyfill_contents().unwrap().as_bytes())
            .await
            .unwrap();
        p.finish_with_message(format!("{} Compiled luau files", "[+]".green()));
        if let Some(modules) = &self.used_modules {
            let inside = modules.lock().unwrap();
            Ok(inside
                .iter()
                .map(|x| Modules::from_str(&uppercase_first(&x)))
                .filter_map(Result::ok)
                .collect())
        } else {
            Ok(vec![])
        }
    }
}

fn format_option<T: ToString>(value: Option<T>) -> String {
    value.map(|x| x.to_string()).unwrap_or("nil".to_string())
}

pub async fn build(path: Option<PathBuf>, run: Strategy, one_file: bool) -> anyhow::Result<()> {
    let root = relative(path);
    if !root.join("kaledis.toml").exists() {
        println!("{}", "No Project found!".red());
        return Ok(());
    }

    let configs = Config::from_toml_file(root.join("kaledis.toml"))?;

    if configs.project.name.len() < 1 {
        eprintln!("{}", "Cannot distribute a game without a name".red());
        return Ok(());
    }

    let mut builder = Builder::new(root, configs, run.clone(), one_file).await?;
    builder.clean_build_folder().await?;

    let imported_modules = builder.add_luau_files().await?;

    if !builder.paths.root.join("conf.luau").exists()
        || builder.paths.root.join("conf.lua").exists()
    {
        builder.make_conf_file(imported_modules).await;
    }

    match run {
        Strategy::BuildDev => {
            builder.add_assets().await;
        }
        Strategy::BuildAndCompile => {
            // create the zip
            let mut zip = Zipper::new();
            zip.put_folder_recursively(builder.paths.build.clone(), None)?;
            if let Some(assets_path) = &builder.paths.assets {
                zip.put_folder_recursively(assets_path.clone(), Some(PathBuf::from("assets")))?;
            }
            builder.clean_build_folder().await?;
            let game_data = zip.finish();

            remove_dir_all(builder.paths.dist.clone()).await.unwrap();
            let new_exe = builder
                .paths
                .dist
                .join(builder.config.project.name)
                .with_extension("exe");
            {
                let mut contents =
                    File::open(builder.config.project.love_path.join("love.exe")).await?;
                let mut output = File::create(&new_exe).await?;
                tokio::io::copy(&mut contents, &mut output).await?;
                tokio::io::copy(&mut &game_data[..], &mut output).await?;
            }

            println!("Saving executable in : {}", new_exe.display().to_string());

            macro_rules! import_love_file {
                ($name:expr) => {
                    {
                        let path = builder.config.project.love_path.join($name);
                        if path.exists() {
                            std::fs::copy(&path, &builder.paths.dist.join($name))?;
                        } else {
                            println!("{}{:?}", "Missing file: ".red(), path);
                        }
                    }
                };
                ($name:expr, $($na:expr),+) => {
                    import_love_file!($name);
                    import_love_file!($($na), +)
                };
            }
            remove_dir_all(&builder.paths.build).await.unwrap();
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
        }
        Strategy::Build => {
            let mut zip = Zipper::new();
            zip.put_folder_recursively(builder.paths.build.clone(), None)?;
            if let Some(assets_path) = &builder.paths.assets {
                zip.put_folder_recursively(assets_path.clone(), Some(PathBuf::from("assets")))?;
            }
            builder.clean_build_folder().await?;
            let game_data = zip.finish();

            let mut file = File::create(builder.paths.build.join("final.love")).await?;
            file.write(&game_data).await?;
        }
    }

    println!("{} {}", "[+]".green(), "Love project builded sucessfully");

    Ok(())
}
