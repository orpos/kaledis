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
use std::io;
use std::path::PathBuf;
use tokio::fs;
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    input: PathBuf,
    removes: Option<Vec<String>>,
    settings: IndexMap<String, bool>,
    libraries: IndexMap<String, bool>,
}

impl Config {
    pub fn input(&self) -> &PathBuf {
        &self.input
    }

    pub fn settings(&self) -> &IndexMap<String, bool> {
        &self.settings
    }

    pub fn libraries(&self) -> &IndexMap<String, bool> {
        &self.libraries
    }

    pub fn removes(&self) -> &Option<Vec<String>> {
        &self.removes
    }
}

impl Config {
    pub async fn from_file(path: impl Into<PathBuf>) -> Result<Self> {
        let content = fs::read_to_string(path.into()).await?;

        Ok(toml::from_str(content.as_str())?)
    }
}

pub struct Polyfill {
    path: PathBuf,
    repository: Repository,
    config: Config,
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

    let path = dirs::cache_dir()
        .ok_or_else(|| anyhow!("could not find cache directory"))?
        .join("dal")
        .join("polyfills")
        .join(ident);

    Ok(path)
}

impl Polyfill {
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

        let config = Config::from_file(path.join("config.toml")).await?;

        Ok(Self {
            path,
            repository,
            config,
        })
    }

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

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}
