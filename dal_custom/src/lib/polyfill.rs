use anyhow::Result;
use anyhow::{anyhow, Context};
use auth_git2::GitAuthenticator;
use blake3;
use dirs;
use fs_err;
use git2::Repository;
use hex;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use tokio::fs;
use url::Url;

use crate::manifest::WritableManifest;
use crate::{utils, TargetVersion};

/// Cleans cache from polyfill repository url.
pub async fn clean_cache(url: &Url) -> Result<()> {
    let index_path = index_path(url)?;
    fs::remove_dir_all(index_path).await?;
    Ok(())
}

/// Cleans every caches of polyfill.
pub async fn clean_cache_all() -> Result<()> {
    let path = cache_dir()?;
    fs::remove_dir_all(path).await?;
    Ok(())
}

/// Gets cache directory path of polyfills.
pub fn cache_dir() -> Result<PathBuf> {
    Ok(dirs::cache_dir()
        .ok_or_else(|| anyhow!("could not find cache directory"))?
        .join("dal")
        .join("polyfills"))
}

/// Polyfill's manifest (`/polyfill.toml` in a polyfill repository)
#[derive(Debug, Deserialize, Serialize)]
pub struct Manifest {
    globals: PathBuf,
    removes: Option<Vec<String>>,
    config: IndexMap<String, bool>,
    lua_version: TargetVersion,
}

impl WritableManifest for Manifest {}

/// Polyfill's globals.
#[derive(Debug)]
pub struct Globals {
    path: PathBuf,
    exports: HashSet<String>,
}

impl Globals {
    #[inline]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    #[inline]
    pub fn exports(&self) -> &HashSet<String> {
        &self.exports
    }
}

/// Represents a polyfill structure.
pub struct Polyfill {
    repository: Repository,
    path: PathBuf,
    globals: Globals,
    removes: Option<Vec<String>>,
    config: IndexMap<String, bool>,
}

fn index_path(url: &Url) -> anyhow::Result<PathBuf> {
    let name = match (url.domain(), url.scheme()) {
        (Some(domain), _) => domain,
        (None, "file") => "local",
        _ => "unknown",
    };

    let hash = blake3::hash(url.to_string().as_bytes());
    let hash_hex = hex::encode(&hash.as_bytes()[..8]);
    let ident = format!("{}-{}", name, hash_hex);

    let path = cache_dir()?
        .join(ident);

    log::debug!("index path {:?}", path);

    Ok(path)
}

impl Polyfill {
    /// Creates a new polyfill from git repository.
    pub async fn new(url: &Url) -> Result<Self> {
        let path = index_path(url)?;
        let repository = match Repository::open(path.as_path()) {
            Ok(repo) => repo,
            Err(_) => {
                if let Err(err) = fs_err::remove_dir_all(path.as_path()) {
                    if err.kind() != io::ErrorKind::NotFound {
                        return Err(err.into());
                    }
                }

                fs_err::create_dir_all(path.as_path())?;
                let auth = GitAuthenticator::new();
                auth.clone_repo(url, &path.as_path())?
            }
        };

        log::info!("repository is ready");

        //let manifest = Manifest::from_file(path.join("polyfill.toml")).await?;
        let manifest_content = fs::read_to_string(path.join("polyfill.toml")).await?;
        let manifest: Manifest = toml::from_str(&manifest_content)?;

        let globals_path = path.join(&manifest.globals);
        log::debug!("globals path {:?}", globals_path);
        let globals_ast = utils::parse_file(&globals_path, &manifest.lua_version).await?;
        let exports = utils::get_exports_from_last_stmt(&utils::ParseTarget::FullMoonAst(globals_ast))
            .await?
            .ok_or_else(|| anyhow!("Invalid polyfill structure. Polyfills' globals must return at least one global in a table."))?;

        let globals = Globals {
            path: globals_path,
            exports,
        };

        log::info!("polyfill ready");

        Ok(Self {
            path,
            repository,
            globals: globals,
            removes: manifest.removes,
            config: manifest.config,
        })
    }

    /// Fetches and updates polyfill repository using git.
    pub fn fetch(&self) -> Result<()> {
        let mut remote = self.repository.find_remote("origin")?;
        let auth = GitAuthenticator::new();
        auth.fetch(&self.repository, &mut remote, &["main"], None)
            .with_context(|| format!("Could not fetch git repository"))?;

        let mut options = git2::build::CheckoutBuilder::new();
        options.force();

        let commit = self
            .repository
            .find_reference("FETCH_HEAD")?
            .peel_to_commit()?;
        self.repository
            .reset(
                &commit.into_object(),
                git2::ResetType::Hard,
                Some(&mut options),
            )
            .with_context(|| format!("Could not reset git repo to fetch_head"))?;

        Ok(())
    }

    #[inline]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    #[inline]
    pub fn globals(&self) -> &Globals {
        &self.globals
    }

    #[inline]
    pub fn removes(&self) -> &Option<Vec<String>> {
        &self.removes
    }

    #[inline]
    pub fn config(&self) -> &IndexMap<String, bool> {
        &self.config
    }
}
