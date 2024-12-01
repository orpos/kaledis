use std::{path::PathBuf, str::FromStr};

use anyhow::{anyhow, Result};
use darklua_core::{
    rules::{self, bundle::BundleRequireMode},
    BundleConfiguration, Configuration, GeneratorParameters, Options, Resources,
};
use full_moon::ast::Ast;
use indexmap::IndexMap;
use tokio::fs;

use crate::{
    injector::Injector, manifest::Manifest, modifiers::Modifier, polyfill::Polyfill, utils,
    TargetVersion,
};

pub const DAL_GLOBAL_IDENTIFIER_PREFIX: &str = "DAL_";
pub const DEFAULT_INJECTED_POLYFILL_NAME: &str = "__polyfill__";

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
    "remove_spaces",
    "remove_nil_declaration",
    "remove_function_call_parens",
    "remove_comments",
    "convert_index_to_field",
    "compute_expression",
    "filter_after_early_return",
    "remove_unused_variable",
    "remove_unused_while",
    "remove_unused_if_branch",
    "remove_empty_do",
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

/// A transpiler that transforms luau to lua
pub struct Transpiler {
    modifiers: IndexMap<String, bool>,
    polyfill: Option<Polyfill>,
    extension: Option<String>,
    target_version: TargetVersion,
    injected_polyfill_name: Option<String>,
}

impl Default for Transpiler {
    fn default() -> Self {
        Self {
            modifiers: default_modifiers_index(),
            polyfill: None,
            extension: None,
            target_version: TargetVersion::Default,
            injected_polyfill_name: None,
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

    pub fn with_polyfill(
        mut self,
        polyfill: Polyfill,
        new_injected_polyfill_name: Option<String>,
    ) -> Self {
        self.polyfill = Some(polyfill);
        self.injected_polyfill_name = new_injected_polyfill_name;
        self
    }

    #[inline]
    async fn parse_file(&self, path: &PathBuf) -> Result<Ast> {
        utils::parse_file(path, &self.target_version).await
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

    pub async fn process(&self, input: PathBuf, output: PathBuf, additional_modifiers: Option<&mut Vec<Modifier>>, bundle: bool) -> Result<()> {
        let output_files = self.private_process(input, output, additional_modifiers, bundle).await?;
        if let Some(polyfill) = &self.polyfill {
            let polyfill_config = polyfill.config();

            // needed additional modifiers: inject_global_value
            let mut additional_modifiers: Vec<Modifier> = Vec::new();
            for (key, value) in polyfill_config {
                let mut identifier = DAL_GLOBAL_IDENTIFIER_PREFIX.to_string();
                identifier.push_str(key);
                let inject_global_value = rules::InjectGlobalValue::boolean(identifier, *value);
                additional_modifiers.push(Modifier::DarkluaRule(Box::new(inject_global_value)));
            }

            if let Some(first_output) = output_files.first() {
				log::debug!("first output found!");
                let extension = if let Some(extension) = &self.extension {
                    extension.to_owned()
                } else {
                    first_output
                        .extension()
                        .ok_or_else(|| anyhow!("Failed to get extension from output file."))?
                        .to_string_lossy()
                        .into_owned()
                };

                if let Some(module_path) = first_output.parent().map(|parent| {
                    parent
                        .join(
                            if let Some(injected_polyfill_name) = &self.injected_polyfill_name {
                                injected_polyfill_name
                            } else {
                                DEFAULT_INJECTED_POLYFILL_NAME
                            },
                        )
                        .with_extension(extension)
                }) {
                    let _ = self
                        .private_process(
                            polyfill.globals().path().to_path_buf(),
                            module_path.to_owned(),
                            Some(&mut additional_modifiers),
                            true,
                        )
                        .await?;

					log::info!("[injector] exports to inject: {:?}", polyfill.globals().exports().to_owned());

                    let injector = Injector::new(
                        module_path,
                        polyfill.globals().exports().to_owned(),
                        self.target_version.to_lua_version(),
                        polyfill.removes().to_owned(),
                    );

                    for source_path in &output_files {
                        injector.inject(source_path).await?
                    }
                }
            }

            Ok(())
        } else {
            Err(anyhow!("Polyfill not found."))
        }
    }
}
