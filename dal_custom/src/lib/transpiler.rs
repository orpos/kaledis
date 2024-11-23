use std::{
    borrow::Cow,
    collections::HashSet,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, Result};
use darklua_core::{
    rules::{self, bundle::BundleRequireMode},
    BundleConfiguration, Configuration, GeneratorParameters, Options, Resources,
};
use full_moon::{
    ast::Ast,
    tokenizer::{Token, TokenType},
    visitors::Visitor,
};
use indexmap::IndexMap;
use path_slash::PathBufExt;
use pathdiff::diff_paths;
use tokio::fs;

use crate::{manifest::Manifest, modifiers::Modifier, polyfill::Polyfill, TargetVersion};

pub const DAL_GLOBAL_IDENTIFIER_PREFIX: &str = "DAL_";
pub const DEFAULT_INJECTED_LIB_FILE_NAME: &str = "__dal_libs__";

pub const DEFAULT_LUAU_TO_LUA_MODIFIERS: [&str; 8] = [
    "remove_interpolated_string",
    "remove_compound_assignment",
    "remove_types",
    "remove_if_expression",
    "remove_continue",
    "remove_redeclared_keys",
    "remove_generalized_iteration",
    "remove_number_literals",
];

pub const DEFAULT_MINIFYING_MODIFIERS: [&str; 11] = [
    "remove_unused_variable",
    "remove_unused_while",
    "remove_unused_if_branch",
    "remove_spaces",
    "remove_nil_declaration",
    "remove_function_call_parens",
    "remove_empty_do",
    "remove_comments",
    "convert_index_to_field",
    "compute_expression",
    "filter_after_early_return",
];

#[inline]
fn modifiers_from_index(modifiers: &IndexMap<String, bool>) -> Result<Vec<Modifier>> {
    modifiers
        .iter()
        .filter_map(|(key, &value)| {
            if value {
                Some(Modifier::from_str(key.as_str()))
            } else {
                None
            }
        })
        .collect()
}

#[inline]
fn default_modifiers_index() -> IndexMap<String, bool> {
    let mut modifiers: IndexMap<String, bool> = IndexMap::new();
    for name in DEFAULT_LUAU_TO_LUA_MODIFIERS {
        modifiers.insert(name.to_owned(), true);
    }
    modifiers
}

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
    libraries_option: IndexMap<String, bool>,
    used_libraries: HashSet<String>,
}

impl CollectUsedLibraries {
    fn new(libraries_option: IndexMap<String, bool>) -> Self {
        Self {
            libraries_option,
            used_libraries: HashSet::new(),
        }
    }

    fn think(&mut self, identifier: String) {
        if let Some(&enabled) = self.libraries_option.get(&identifier) {
            if enabled {
                self.used_libraries.insert(identifier);
            }
        }
    }
}

impl Visitor for CollectUsedLibraries {
    fn visit_identifier(&mut self, identifier: &Token) {
        if let TokenType::Identifier { identifier } = identifier.token_type() {
            self.think(identifier.to_string());
        }
    }
}

/// A transpiler that transforms luau to lua
pub struct Transpiler {
    modifiers: IndexMap<String, bool>,
    polyfill: Option<Polyfill>,
    extension: Option<String>,
    target_version: TargetVersion,
}

impl Default for Transpiler {
    fn default() -> Self {
        Self {
            modifiers: default_modifiers_index(),
            polyfill: None,
            extension: None,
            target_version: TargetVersion::Default,
        }
    }
}

impl Transpiler {
    pub fn with_minifying_modifiers(mut self) -> Self {
        for name in DEFAULT_MINIFYING_MODIFIERS {
            self.modifiers.insert(name.to_owned(), true);
        }
        self
    }

    pub fn with_modifiers(mut self, modifiers: &IndexMap<String, bool>) -> Self {
        for (key, value) in modifiers {
            let value = if let Some(&default_value) = self.modifiers.get(key) {
                default_value && *value
            } else {
                *value
            };
            self.modifiers.insert(key.to_owned(), value);
        }
        self
    }

    pub fn with_manifest(mut self, manifest: &Manifest) -> Self {
        self = self.with_modifiers(manifest.modifiers());
        if manifest.minify {
            self = self.with_minifying_modifiers();
        }
        if let Some(extension) = manifest.extension() {
            self = self.with_extension(extension);
        }
        self.target_version = manifest.target_version().clone();
        self
    }

    pub fn with_extension(mut self, extension: impl Into<String>) -> Self {
        self.extension = Some(extension.into());
        self
    }

    pub fn with_polyfill(mut self, polyfill: Polyfill) -> Self {
        self.polyfill = Some(polyfill);
        self
    }

    async fn parse_file(&self, path: &PathBuf) -> Result<Ast> {
        let code = fs::read_to_string(&path).await?;
        let ast = full_moon::parse_fallible(
            code.as_str(),
            (&self.target_version).to_lua_version().clone(),
        )
        .into_result()
        .map_err(|errors| anyhow!("full_moon parsing error: {:?}", errors))?;

        Ok(ast)
    }

    pub async fn inject_library(
        &self,
        file_path: &PathBuf,
        module_path: &PathBuf,
        libraries: &IndexMap<String, bool>,
        removes: &Option<Vec<String>>,
    ) -> Result<()> {
        let parent = file_path
            .parent()
            .ok_or(anyhow!("File path must have parent path"))?;
        let require_path = diff_paths(module_path, parent)
            .ok_or(anyhow!("Couldn't resolve the require path"))?
            .with_extension("");
        let require_path = make_relative(&require_path).to_path_buf();

        let code = fs::read_to_string(file_path).await?;

        let mut lines: Vec<String> = code.lines().map(String::from).collect();
        let mut libraries_texts: Vec<String> = Vec::new();

        let ast = full_moon::parse_fallible(
            code.as_str(),
            (&self.target_version).to_lua_version().clone(),
        )
        .into_result()
        .map_err(|errors| anyhow!("{:?}", errors))?;

        let mut collect_used_libs = CollectUsedLibraries::new(libraries.clone());
        collect_used_libs.visit_ast(&ast);

        for lib in collect_used_libs.used_libraries {
            libraries_texts.push(format!(
                "local {}=require'{}'.{} ",
                lib,
                require_path.to_slash_lossy(),
                lib
            ));
        }

        if let Some(removes) = removes {
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

        fs::write(file_path, new_content).await?;

        Ok(())
    }

    async fn private_process(
        &self,
        input: PathBuf,
        output: PathBuf,
        additional_modifiers: Option<&mut Vec<Modifier>>,
        bundle: bool,
    ) -> Result<Vec<PathBuf>> {
        let resources = Resources::from_file_system();

        let mut modifiers = Vec::new();
        if let Some(additional_modifiers) = additional_modifiers {
            modifiers.append(additional_modifiers);
        }
        modifiers.append(&mut modifiers_from_index(&self.modifiers)?);

        let (rules, mut fullmoon_visitors) = modifiers.into_iter().fold(
            (Vec::new(), Vec::new()),
            |(mut rules, mut fullmoon_visitors), modifier| {
                match modifier {
                    Modifier::DarkluaRule(darklua_rule) => rules.push(darklua_rule),
                    Modifier::FullMoonVisitor(fullmoon_visitor) => {
                        fullmoon_visitors.push(fullmoon_visitor)
                    }
                }
                (rules, fullmoon_visitors)
            },
        );

        let mut options = Options::new(input).with_configuration({
            // let mut config: Configuration = if bundle {
            //     toml::from_str("bundle = { require_mode = 'path' }").unwrap()
            // } else {
            //     Configuration::empty()
            // };
            let mut config = Configuration::empty();

            if bundle {
                config = config.with_bundle_configuration(BundleConfiguration::new(
                    BundleRequireMode::default(),
                ));
            }
            config = config.with_generator(GeneratorParameters::RetainLines);

            rules
                .into_iter()
                .fold(config, |config, rule| config.with_rule(rule))
        });
        options = options.with_output(&output);
        let result = darklua_core::process(&resources, options);

        let success_count = result.success_count();
        if result.has_errored() {
            let error_count = result.error_count();
            eprintln!(
                "{}{} error{} happened:",
                if success_count > 0 { "but " } else { "" },
                error_count,
                if error_count > 1 { "s" } else { "" }
            );

            result.errors().for_each(|error| eprintln!("-> {}", error));

            return Err(anyhow!("darklua process was not successful"));
        }

        let mut created_files: Vec<PathBuf> = result.into_created_files().collect();
        let extension = &self.extension;
        if fullmoon_visitors.is_empty() {
            if let Some(extension) = extension {
                for path in &mut created_files {
                    let old_path = path.clone();
                    path.set_extension(extension);
                    fs::rename(old_path, path).await?;
                }
            }
        } else {
            for path in &mut created_files {
                let mut ast = self.parse_file(&path).await?;

                for visitor in &mut fullmoon_visitors {
                    ast = visitor.visit_ast_boxed(ast);
                }

                if let Some(extension) = extension {
                    let old_path = path.clone();
                    path.set_extension(extension);
                    let new_path = path.to_owned();
                    if new_path != old_path && old_path.exists() {
                        fs::remove_file(old_path).await?;
                    }
                }

                fs::write(path, ast.to_string()).await?;
            }
        }

        Ok(created_files)
    }

    pub async fn process(&self, input: PathBuf, output: PathBuf) -> Result<()> {
        let output_files = self.private_process(input, output, None, false).await?;
        if let Some(polyfill) = &self.polyfill {
            let polyfill_config = polyfill.config();
            let polyfill_path = polyfill.path();

            // needed additional modifiers: inject_global_value
            let mut additional_modifiers: Vec<Modifier> = Vec::new();
            for (key, value) in polyfill_config
                .settings()
                .iter()
                .chain(polyfill_config.libraries().iter())
            {
                let mut identifier = DAL_GLOBAL_IDENTIFIER_PREFIX.to_string();
                identifier.push_str(key);
                let inject_global_value = rules::InjectGlobalValue::boolean(identifier, *value);
                additional_modifiers.push(Modifier::DarkluaRule(Box::new(inject_global_value)));
            }

            for (index, output_path) in output_files.iter().enumerate() {
                if let Some(parent) = output_path.parent() {
                    let mut module_path: Option<PathBuf> = None;
                    if index == 0 {
                        let extension = if let Some(extension) = &self.extension {
                            extension.to_owned()
                        } else {
                            output_path
                                .extension()
                                .unwrap()
                                .to_string_lossy()
                                .into_owned()
                        };
                        module_path = Some(
                            parent
                                .join(DEFAULT_INJECTED_LIB_FILE_NAME)
                                .with_extension(extension),
                        );
                        let _ = self
                            .private_process(
                                polyfill_path.join(polyfill_config.input()),
                                module_path.to_owned().unwrap(),
                                Some(&mut additional_modifiers),
                                true,
                            )
                            .await?;
                    }
                    if let Some(module_path) = module_path {
                        self.inject_library(
                            &output_path,
                            &module_path,
                            polyfill_config.libraries(),
                            polyfill_config.removes(),
                        )
                        .await?;
                    }
                }
            }
        }
        Ok(())
    }
}
