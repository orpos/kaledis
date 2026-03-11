pub mod android;
pub mod build_utils;
pub mod linux;
pub mod macos;
pub mod windows;

use std::{
    io::{BufRead, BufReader, Cursor, Read, Write},
    path::PathBuf,
    process::exit,
    str::FromStr,
};

use backhand::{FilesystemReader, FilesystemWriter, InnerNode, kind::Kind};
use color_eyre::Section;
use colored::Colorize;
use fs_err::tokio::{
    File, canonicalize, copy, create_dir, create_dir_all, hard_link, remove_dir_all, remove_file,
    rename,
};
use indicatif::{MultiProgress, ProgressBar};
use strum::IntoEnumIterator;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use walkdir::WalkDir;

use crate::{
    commands::build::{
        android::build_android, linux::build_linux, macos::build_macos, windows::build_windows,
    },
    dalbit::{
        manifest::Manifest,
        transpile::{clean_polyfill, process_files},
    },
    home_manager::{HomeManager, Target},
    toml_conf::{KaledisConfig, LoveConfig, Modules},
    utils::relative,
    zip_utils::Zipper,
};
use build_utils::{Paths, get_transpiler, read_aliases};

#[derive(PartialEq, Eq, Clone)]
pub enum Strategy {
    /// Just creates the love file
    Build(Vec<Target>),
    /// Just compiles the lua files
    BuildDev,
}

pub enum LoveCfg {
    File(PathBuf),
    Config(LoveConfig),
}

pub struct Builder {
    pub config: KaledisConfig,
    pub love_config: LoveCfg,
    pub strategy: Strategy,
    pub aliases: Vec<(String, String)>,
    pub progress_bar: MultiProgress,
    // Version management
    pub home: HomeManager,
    // Paths management (utils)
    pub paths: Paths,
    pub bundle: bool,
    pub manifest: Manifest,
}

impl Builder {
    pub async fn get_love_config(root: &PathBuf) -> LoveCfg {
        if root.join("conf.luau").exists() {
            return LoveCfg::File(root.join("conf.luau"));
        } else if root.join("conf.toml").exists() {
            return LoveCfg::Config(LoveConfig::from_toml_file(root.join("conf.toml")).expect(
                "Failed to get love config, you should use either conf.luau or conf.toml.",
            ));
        }
        dbg!(root);
        tracing::error!("No config file found!");
        exit(1)
    }

    pub async fn new(root: PathBuf, strategy: Strategy, bundle: bool) -> Self {
        let love_config = Self::get_love_config(&root).await;
        let config = KaledisConfig::from_toml_file(root.join("kaledis.toml")).expect("Love Config");

        return Self::from_configs(root, config, love_config, strategy, bundle).await;
    }

    pub async fn from_configs(
        root: PathBuf,
        config: KaledisConfig,
        love_config: LoveCfg,
        strategy: Strategy,
        bundle: bool,
    ) -> Self {
        clean_polyfill();
        let manager = HomeManager::new().await.unwrap();
        match &strategy {
            Strategy::Build(targets) => {
                for target in targets {
                    if *target == Target::LoveFile {
                        continue;
                    }
                    manager.ensure_version(&config.love, target.clone()).await;
                }
            }
            Strategy::BuildDev => {
                #[cfg(windows)]
                manager.ensure_version(&config.love, Target::Windows).await;
                #[cfg(target_os = "linux")]
                manager
                    .ensure_version(&config.love, Target::LinuxAppImage)
                    .await;
                #[cfg(target_os = "macos")]
                manager.ensure_version(&config.love, Target::Macos).await;
            }
        }

        Self {
            manifest: get_transpiler(bundle, config.polyfill.as_ref())
                .await
                .expect("Failed to build manifest"),
            love_config: love_config,
            aliases: read_aliases(&root).await.expect("Failed to read aliases"),
            paths: Paths::from_root(root, &config),
            config: config,
            home: manager,
            progress_bar: MultiProgress::new(),
            strategy,
            bundle,
        }
    }

    /// Build steps
    pub async fn clean_build_folder(&self) -> color_eyre::Result<()> {
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
    pub async fn add_assets(&self, zipper: Option<&mut Zipper>, finishing_love: bool) {
        let mut to_link = self.config.layout.external.clone();

        let mut p = ProgressBar::new_spinner().with_message("Adding assets...");
        p = self.progress_bar.add(p);

        if self.strategy == Strategy::BuildDev || finishing_love {
            if !finishing_love {
                to_link.extend_from_slice(&self.config.layout.bundle);
            }
            for glb in &to_link {
                for path in glob::glob(&self.paths.root.join(glb).to_string_lossy())
                    .unwrap()
                    .filter_map(Result::ok)
                {
                    let pth_b = &self.paths.build.join(
                        &path
                            .strip_prefix(&self.paths.root)
                            .suggestion("Don't use assets outside the root of your project")
                            .expect("Failed to remove root"),
                    );
                    create_dir_all(&pth_b.parent().expect("Invalid path"))
                        .await
                        .expect("Failed to create file structure");
                    if pth_b.exists() {
                        remove_file(&pth_b)
                            .await
                            .expect("Failed to clean previous asset");
                    }
                    hard_link(&path, &pth_b)
                        .await
                        .expect("Failed to link the file");
                }
            }
        } else if let Some(zipper) = zipper {
            for glb in &self.config.layout.bundle {
                for path in glob::glob(&self.paths.root.join(glb).to_string_lossy())
                    .unwrap()
                    .filter_map(Result::ok)
                {
                    zipper.add_rootless(&path, &self.paths.root).unwrap();
                }
            }
        }
        p.finish_with_message(format!("{} Assets Added", "[+]".green()));
    }

    pub async fn handle_conf_file(&self, used_modules: Vec<Modules>) {
        match &self.love_config {
            LoveCfg::Config(cfg) => {
                let contents;
                if let Strategy::BuildDev = self.strategy {
                    contents = cfg.to_string(Modules::iter().collect());
                } else {
                    contents = cfg.to_string(used_modules);
                }
                let mut file = File::create(self.paths.build.join("conf.lua"))
                    .await
                    .expect("failed to create conf.lua file");

                file.write_all(&contents.as_bytes())
                    .await
                    .expect("Failed to write conf.lua")
            }
            LoveCfg::File(file) => {
                self._transpile_files(file, &self.paths.build.join("conf.lua"))
                    .await;
            }
        }
    }

    pub async fn _transpile_files(&self, input: &PathBuf, output: &PathBuf) -> Vec<Modules> {
        let mut p = ProgressBar::new_spinner().with_message("Building...");
        p = self.progress_bar.add(p);
        let mut new_manifest = self.manifest.clone();
        new_manifest.hmr = self.strategy == Strategy::BuildDev && self.config.hmr;

        let mut used_modules = process_files(
            &new_manifest,
            input,
            output,
            self.paths.clone(),
            self.config.detect_modules,
            &self.aliases,
            self.bundle.clone(),
        )
        .expect("Failed to process luau files");

        if !self.config.detect_modules
            && let LoveCfg::Config(cfg) = &self.love_config
        {
            used_modules.extend_from_slice(&cfg.modules);
        }

        p.finish_with_message(format!(
            "{} Built {}",
            "[+]".green(),
            input
                .file_name()
                .map(|x| x.to_string_lossy().to_string())
                .unwrap_or(String::new())
        ));
        used_modules
    }

    // love2d require function is a little different than the normal one
    pub async fn rename_dots_to_underscores(&self) -> io::Result<()> {
        // We collect entries into a vector first to avoid "path not found"
        // errors while the iterator is still running.
        let mut entries: Vec<_> = WalkDir::new(&self.paths.build)
            // Exclude the root foolder
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .collect();

        // Sort by depth in reverse (bottom-up).
        // This ensures we rename files inside a folder BEFORE renaming the folder itself.
        entries.sort_by(|a, b| b.depth().cmp(&a.depth()));

        for entry in entries {
            let path = entry.path();

            // Extract the file/folder name
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                // Only proceed if it contains a dot and isn't just "." or ".."
                if path.is_file() {
                    continue;
                }
                if file_name.contains('.') && file_name != "." && file_name != ".." {
                    let new_name = file_name.replace('.', "__");
                    let mut new_path = path.to_path_buf();
                    new_path.set_file_name(new_name);

                    rename(path, new_path).await?;
                }
            }
        }

        Ok(())
    }

    pub async fn transpile(&self) -> Vec<Modules> {
        let mut result = vec![];

        for (_, path) in &self.aliases {
            let bas;
            if path.starts_with(".") {
                bas = canonicalize(self.paths.root.join(path))
                    .await
                    .expect("Failed to resolve path from alias");
            } else {
                bas = self
                    .paths
                    .root
                    .join(PathBuf::from_str(&path).expect("Failed to parse path"));
            }
            result.extend_from_slice(
                &self
                    ._transpile_files(&bas, &self.paths.build.join(path))
                    .await,
            );
        }
        result.extend_from_slice(
            &self
                ._transpile_files(
                    &self.paths.root.join(&self.config.layout.code),
                    &self.paths.build,
                )
                .await,
        );

        self.rename_dots_to_underscores()
            .await
            .expect("Failed to rename dots");

        result
    }
}

pub async fn build(path: Option<PathBuf>, run: Strategy, bundle: bool) -> color_eyre::Result<()> {
    let root = relative(path);
    if !root.join("kaledis.toml").exists() {
        panic!("Not a valid kaledis project");
    }

    let builder = Builder::new(root.clone(), run.clone(), bundle).await;
    builder
        .clean_build_folder()
        .await
        .expect("Failed to clean build folder");
    builder.handle_conf_file(builder.transpile().await).await;

    match run {
        Strategy::BuildDev => {
            builder.add_assets(None, false).await;
            builder.transpile().await;
        }
        Strategy::Build(platforms) => {
            let mut zip = Zipper::new();
            builder.add_assets(Some(&mut zip), false).await;
            zip.put_folder_recursively(&builder.paths.build)
                .expect("Failed to create zip");
            let data = zip.finish();

            for platform in platforms {
                // We skip when we use love file because it basically is done at this state
                if let Target::LoveFile = platform {
                    remove_dir_all(&builder.paths.build)
                        .await
                        .expect("Failed to clean build folder");
                    create_dir_all(&builder.paths.build)
                        .await
                        .expect("Failed to create build folder");
                    let mut file = File::create(builder.paths.build.join("final.love")).await?;
                    file.write_all(&data).await?;

                    builder.add_assets(None, true).await;

                    continue;
                }

                let home = &builder.home;
                home.ensure_version(&builder.config.love, platform.clone())
                    .await;

                let platform_path = home.get_path(&builder.config.love, platform.clone()).await;

                if platform != Target::Android {
                    let dists = builder.paths.dist.join(platform.as_ref().to_string());
                    if dists.exists() {
                        remove_dir_all(&dists)
                            .await
                            .expect("Failed to clean folder");
                    }
                    create_dir_all(&dists)
                        .await
                        .expect("Failed to create output folder");

                    recursive_copy(&platform_path, &dists)
                        .await
                        .expect("Failed to copy files");
                }

                match platform {
                    Target::LoveFile => {}
                    Target::Android => {
                        build_android(&builder, &data)
                            .await
                            .expect("Failed to start android server");
                    }
                    Target::LinuxAppImage => {
                        build_linux(&builder, &data).await?;
                    }
                    Target::Macos => {
                        build_macos(&builder, &data).await?;
                    }
                    Target::Windows => {
                        build_windows(&builder, &data)
                            .await
                            .expect("Failed to build to windows");
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn recursive_copy(input: &PathBuf, output: &PathBuf) -> color_eyre::Result<()> {
    for entry in WalkDir::new(&input).into_iter().filter_map(Result::ok) {
        let from = entry.path();
        let to = output.join(from.strip_prefix(&input)?);

        // create directories
        if entry.file_type().is_dir() {
            if let Err(e) = create_dir(to).await {
                match e.kind() {
                    io::ErrorKind::AlreadyExists => {}
                    _ => return Err(e.into()),
                }
            }
        }
        // copy files
        else if entry.file_type().is_file() {
            copy(from, to).await?;
        }
    }
    Ok(())
}
