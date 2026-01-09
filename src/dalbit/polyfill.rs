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
use url::Url;

use super::utils;
use crate::dalbit::TargetVersion;

pub const DEFAULT_REPO_URL: &str = "https://github.com/orpos/love2d-dalbit-polyfill";
pub const DEFAULT_INJECTION_PATH: &str = "__polyfill__";

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
}

/// Polyfill's manifest (`/polyfill.toml` in a polyfill repository)
#[derive(Debug, Deserialize, Serialize)]
pub struct PolyfillManifest {
    globals: PathBuf,
    removes: Option<Vec<String>>,
    config: HashMap<String, bool>,
    lua_version: TargetVersion,
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
