use anyhow::{Context, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::{polyfill::Polyfill, TargetVersion};


/// Manifest for dalbit transpiler. This is a writable manifest.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Manifest {
    pub input: PathBuf,
    pub output: PathBuf,
    pub file_extension: Option<String>,
    pub target_version: TargetVersion,
    pub minify: bool,
    pub modifiers: IndexMap<String, bool>,
    pub polyfill: Option<Polyfill>,
    pub bundle: bool
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            input: Path::new("input.luau").to_owned(),
            output: Path::new("output.lua").to_owned(),
            file_extension: Some("lua".to_owned()),
            target_version: TargetVersion::Lua53,
            minify: true,
            modifiers: IndexMap::new(),
            polyfill: Some(Polyfill::default()),
            bundle: false
        }
    }
}

impl Manifest {
    /// Load manifest from file.
    pub async fn from_file(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let manifest = fs::read_to_string(&path).await?;
        let manifest: Manifest = toml::from_str(&manifest)
            .with_context(|| format!("Failed to parse manifest file: {:?}", path))?;
        // manifest.path = path;
        Ok(manifest)
    }

    /// Write manifest to file.
    pub async fn write(&self, path: impl Into<PathBuf>) -> Result<()> {
        fs::write(path.into(), toml::to_string(self)?).await?;
        Ok(())
    }

    #[inline]
    pub fn input(&self) -> &PathBuf {
        &self.input
    }

    #[inline]
    pub fn output(&self) -> &PathBuf {
        &self.output
    }

    #[inline]
    pub fn file_extension(&self) -> &Option<String> {
        &self.file_extension
    }

    #[inline]
    pub fn modifiers(&self) -> &IndexMap<String, bool> {
        &self.modifiers
    }

    #[inline]
    pub fn target_version(&self) -> &TargetVersion {
        &self.target_version
    }

    #[inline]
    pub fn polyfill(&self) -> &Option<Polyfill> {
        &self.polyfill
    }
}