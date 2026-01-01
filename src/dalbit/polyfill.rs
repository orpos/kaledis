use anyhow::Result;
use anyhow::{anyhow, Context};
use auth_git2::GitAuthenticator;
use blake3;
use dirs;
use fs_err;
use git2::Repository;
use hex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::fs;
use url::Url;

use super::utils;
use crate::dalbit::TargetVersion;

pub const DEFAULT_REPO_URL: &str = "https://github.com/CavefulGames/dalbit-polyfill";
pub const DEFAULT_INJECTION_PATH: &str = "__polyfill__";

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
        .join("dalbit")
        .join("polyfills"))
}

/// Polyfill-related manifest.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Polyfill {
    repository: Url,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    globals: HashMap<String, bool>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub config: HashMap<String, bool>,
    injection_path: PathBuf,
}

impl Default for Polyfill {
    fn default() -> Self {
        Self {
            repository: Url::from_str(DEFAULT_REPO_URL).unwrap(),
            globals: HashMap::new(),
            config: HashMap::new(),
            injection_path: PathBuf::from_str(DEFAULT_INJECTION_PATH).unwrap(),
        }
    }
}

impl Polyfill {
    pub fn new(repository: Url, injection_path: PathBuf) -> Self {
        Self {
            repository,
            globals: HashMap::new(),
            config: HashMap::new(),
            injection_path,
        }
    }

    /// Loads polyfill cache.
    pub fn cache(&self) -> Result<PolyfillCache> {
        PolyfillCache::new(&self.repository)
    }
    pub fn cache_sync(&self) -> Result<PolyfillCache> {
        PolyfillCache::new_sync(&self.repository)
    }

    #[inline]
    pub fn repository(&self) -> &Url {
        &self.repository
    }

    #[inline]
    pub fn globals(&self) -> &HashMap<String, bool> {
        &self.globals
    }

    #[inline]
    pub fn config(&self) -> &HashMap<String, bool> {
        &self.config
    }

    #[inline]
    pub fn injection_path(&self) -> &PathBuf {
        &self.injection_path
    }
}

/// Polyfill's manifest (`/polyfill.toml` in a polyfill repository)
#[derive(Debug, Deserialize, Serialize)]
pub struct PolyfillManifest {
    globals: PathBuf,
    removes: Option<Vec<String>>,
    config: HashMap<String, bool>,
    lua_version: TargetVersion,
}

impl PolyfillManifest {
    /// Load polyfill manifest from file.
    pub async fn from_file(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let manifest = fs::read_to_string(&path).await?;
        let manifest: Self = toml::from_str(&manifest)
            .with_context(|| format!("Could not parse polyfill manifest file: {:?}", path))?;
        Ok(manifest)
    }

    /// Write polyfill manifest to file.
    pub async fn write(&self, path: impl Into<PathBuf>) -> Result<()> {
        fs::write(path.into(), toml::to_string(self)?).await?;
        Ok(())
    }
}

/// Polyfill's globals.
#[derive(Debug)]
pub struct Globals {
    pub path: PathBuf,
    pub exports: HashSet<String>,
}

/// Represents a loaded polyfill cache.
pub struct PolyfillCache {
    repository: Option<Repository>,
    path: PathBuf,
    globals: Globals,
    removes: Option<Vec<String>>,
    config: HashMap<String, bool>,
}

pub struct PolyfillCacheInfo {
    pub path: PathBuf,
    pub globals: Globals,
    pub removes: Option<Vec<String>>,
    pub config: HashMap<String, bool>,
}

impl Into<PolyfillCacheInfo> for PolyfillCache {
    fn into(self) -> PolyfillCacheInfo {
        PolyfillCacheInfo {
            config : self.config,
            globals: self.globals,
            path: self.path,
            removes: self.removes,
        }
    }
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

    let path = cache_dir()?.join(ident);

    Ok(path)
}

impl PolyfillCache {
    pub fn new_sync(url: &Url) -> Result<Self> {
        let path = if url.scheme() == "file" {
            url.to_file_path().unwrap()
        } else {
            index_path(url)?
        };
        let repository = if url.scheme() == "file" {
            None
        } else {
            Some(match Repository::open(path.as_path()) {
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
            })
        };

        //let manifest = Manifest::from_file(path.join("polyfill.toml")).await?;
        let manifest_content = std::fs::read_to_string(path.join("polyfill.toml"))?;
        let manifest: PolyfillManifest = toml::from_str(&manifest_content)?;

        let globals_path = path.join(&manifest.globals);
        let globals_ast = utils::parse_file(&globals_path, true)?;
        let exports = utils::get_exports_from_last_stmt(&utils::ParseTarget::FullMoonAst(globals_ast))?
            .ok_or_else(|| anyhow!("Invalid polyfill structure. Polyfills' globals must return at least one global in a table."))?;

        let globals = Globals {
            path: globals_path,
            exports,
        };

        Ok(Self {
            path,
            repository,
            globals: globals,
            removes: manifest.removes,
            config: manifest.config,
        })
    }

    /// Creates a new polyfill from git repository.
    pub fn new(url: &Url) -> Result<Self> {
        let path = if url.scheme() == "file" {
            url.to_file_path().unwrap()
        } else {
            index_path(url)?
        };
        let repository = if url.scheme() == "file" {
            None
        } else {
            Some(match Repository::open(path.as_path()) {
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
            })
        };

        //let manifest = Manifest::from_file(path.join("polyfill.toml")).await?;
        let manifest_content = std::fs::read_to_string(path.join("polyfill.toml"))?;
        let manifest: PolyfillManifest = toml::from_str(&manifest_content)?;

        let globals_path = path.join(&manifest.globals);
        let globals_ast = utils::parse_file(&globals_path, true)?;
        let exports = utils::get_exports_from_last_stmt(&utils::ParseTarget::FullMoonAst(globals_ast))?
            .ok_or_else(|| anyhow!("Invalid polyfill structure. Polyfills' globals must return at least one global in a table."))?;

        let globals = Globals {
            path: globals_path,
            exports,
        };

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
        if self.repository.is_none() {
            return Ok(());
        }

        let repo = self.repository.as_ref().unwrap();

        let mut remote = repo.find_remote("origin")?;
        let auth = GitAuthenticator::new();
        auth.fetch(&repo, &mut remote, &["main"], None)
            .with_context(|| format!("Could not fetch git repository"))?;

        let mut options = git2::build::CheckoutBuilder::new();
        options.force();

        let commit = repo.find_reference("FETCH_HEAD")?.peel_to_commit()?;
        repo.reset(
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
    pub fn globals_path(&self) -> &PathBuf {
        &self.globals.path
    }

    #[inline]
    pub fn globals_exports(&self) -> &HashSet<String> {
        &self.globals.exports
    }

    #[inline]
    pub fn removes(&self) -> &Option<Vec<String>> {
        &self.removes
    }

    #[inline]
    pub fn config(&self) -> &HashMap<String, bool> {
        &self.config
    }
}
