use std::{
    borrow::Cow,
    collections::HashSet,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use full_moon::{
    tokenizer::{Token, TokenType},
    visitors::Visitor,
    LuaVersion,
};
use path_slash::PathBufExt;
use pathdiff::diff_paths;
use tokio::fs;

#[inline]
fn make_relative(path: &PathBuf) -> Cow<Path> {
    if path.starts_with(".") | path.starts_with("..") {
        Cow::Borrowed(path.as_path())
    } else {
        Cow::Owned(Path::new(".").join(path))
    }
}

#[derive(Debug)]
struct CollectUsedLibraries {
    libraries: HashSet<String>,
    used_libraries: HashSet<String>,
}

impl CollectUsedLibraries {
    fn new(libraries: HashSet<String>) -> Self {
        Self {
            libraries,
            used_libraries: HashSet::new(),
        }
    }
}

impl Visitor for CollectUsedLibraries {
    fn visit_identifier(&mut self, identifier: &Token) {
        if let TokenType::Identifier { identifier } = identifier.token_type() {
            let identifier = identifier.to_string();
            if self.libraries.contains(&identifier) {
                self.used_libraries.insert(identifier);
            }
        }
    }
}

/// Injector that injects module's export which is a table constructor.
pub struct Injector {
    module_path: PathBuf,
    exports: HashSet<String>,
    removes: Option<Vec<String>>,
    lua_version: LuaVersion,
}

impl Injector {
    pub fn new(
        module_path: PathBuf,
        exports: HashSet<String>,
        lua_version: LuaVersion,
        removes: Option<Vec<String>>,
    ) -> Self {
        Self {
            module_path,
            exports,
            removes,
            lua_version,
        }
    }

    pub fn module_path(&self) -> &PathBuf {
        &self.module_path
    }

    pub fn removes(&self) -> &Option<Vec<String>> {
        &self.removes
    }

    pub async fn inject(&self, source_path: &PathBuf) -> Result<()> {
        let parent = source_path
            .parent()
            .ok_or(anyhow!("File path must have parent path"))?;
        let require_path = diff_paths(self.module_path(), parent)
            .ok_or(anyhow!("Couldn't resolve the require path"))?
            .with_extension("");
        let require_path = make_relative(&require_path).to_path_buf();

        let code = fs::read_to_string(source_path).await?;

        let mut lines: Vec<String> = code.lines().map(String::from).collect();
        let mut libraries_texts: Vec<String> = Vec::new();

        let ast = full_moon::parse_fallible(code.as_str(), self.lua_version)
            .into_result()
            .map_err(|errors| anyhow!("{:?}", errors))?;

        let mut collect_used_libs = CollectUsedLibraries::new(self.exports.clone());
        collect_used_libs.visit_ast(&ast);

        for lib in collect_used_libs.used_libraries {
			log::debug!("used library: {}", lib);
            libraries_texts.push(format!(
                "local {}=require'{}'.{} ",
                lib,
                require_path.to_slash_lossy(),
                lib
            ));
        }

        if let Some(removes) = self.removes() {
            for lib in removes {
                libraries_texts.push(format!("local {}=nil ", lib));
            }
        }

        let libraries_text = libraries_texts.join("");
        if let Some(first_line) = lines.get_mut(0) {
            first_line.insert_str(0, &libraries_text);
        } else {
            lines.push(libraries_text);
        }

        let new_content = lines.join("\n");

		log::debug!("injected source path: {:?}", source_path);

        fs::write(source_path, new_content).await?;

        Ok(())
    }
}
