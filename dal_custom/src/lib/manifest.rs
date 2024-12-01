use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

use crate::TargetVersion;

#[async_trait::async_trait]
pub trait WritableManifest: Send + Sized + Serialize + DeserializeOwned {
    #[inline]
    async fn from_file(path: impl Into<PathBuf> + Send) -> Result<Self> {
        let content = fs::read_to_string(path.into()).await?;

        Ok(toml::from_str(content.as_str())?)
    }

    #[inline]
    async fn write(&self, path: impl Into<PathBuf> + Send) -> Result<()> {
        fs::write(path.into(), toml::to_string(self)?).await?;

        Ok(())
    }
}

/// Manifest for dal transpiler.
#[derive(Debug, Deserialize, Serialize)]
pub struct Manifest {
    pub output: Option<PathBuf>,
    pub input: Option<PathBuf>,
    pub file_extension: Option<String>,
    pub target_version: TargetVersion,
    pub minify: bool,
    pub modifiers: IndexMap<String, bool>,
    pub globals: IndexMap<String, bool>,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            output: None,
            input: None,
            file_extension: Some("lua".to_owned()),
            target_version: TargetVersion::Lua53,
            minify: true,
            // generator: GeneratorParameters::RetainLines,
            modifiers: IndexMap::new(),
            globals: IndexMap::new(),
        }
    }
}

impl WritableManifest for Manifest {}

impl Manifest {
    #[inline]
    pub fn modifiers(&self) -> &IndexMap<String, bool> {
        &self.modifiers
    }

    #[inline]
    pub fn target_version(&self) -> &TargetVersion {
        &self.target_version
    }

    #[inline]
    pub fn extension(&self) -> &Option<String> {
        &self.file_extension
    }

    #[inline]
    pub fn require_input(&self, replacement: Option<PathBuf>) -> Result<PathBuf> {
        replacement
            .or(self.input.clone())
            .ok_or_else(|| anyhow!("Error: 'inputs' is required but not provided."))
    }

    #[inline]
    pub fn require_output(&self, replacement: Option<PathBuf>) -> Result<PathBuf> {
        replacement
            .or(self.output.clone())
            .ok_or_else(|| anyhow!("Error: 'output' is required but not provided."))
    }
}
