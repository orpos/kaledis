use std::{ ffi::OsStr, path::PathBuf, str::FromStr };

use anyhow::{ anyhow, Result };
use async_walkdir::WalkDir;
use darklua_core::{
    rules::{ self, bundle::BundleRequireMode },
    BundleConfiguration,
    Configuration,
    GeneratorParameters,
    Options,
    Resources,
};
use futures_lite::stream::StreamExt;
use indexmap::IndexMap;
use tokio::fs;

use crate::{ injector::Injector, manifest::Manifest, modifiers::Modifier, utils };

pub const DALBIT_GLOBAL_IDENTIFIER_PREFIX: &str = "DALBIT_";

pub const DEFAULT_LUAU_TO_LUA_MODIFIERS: [&str; 9] = [
    "remove_interpolated_string",
    "remove_compound_assignment",
    "remove_types",
    "remove_floor_division",
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

async fn private_process(
    manifest: &Manifest,
    input: &PathBuf,
    output: &PathBuf,
    additional_modifiers: Option<&mut Vec<Modifier>>,
    bundle: bool
) -> Result<Vec<PathBuf>> {
    let resources = Resources::from_file_system();

    let mut modifiers = Vec::new();
    if let Some(additional_modifiers) = additional_modifiers {
        modifiers.append(additional_modifiers);
    }
    {
        let mut transpiling_modifiers = IndexMap::new();
        for name in DEFAULT_LUAU_TO_LUA_MODIFIERS {
            transpiling_modifiers.insert(name, true);
        }
        for (name, enabled) in manifest.modifiers() {
            let name = name.as_str();
            log::debug!("inserted modifier name: {}", name);
            transpiling_modifiers.insert(name, *enabled);
        }
        for (name, enabled) in transpiling_modifiers {
            let modifier = Modifier::from_str(name)?;
            if enabled {
                modifiers.push(modifier);
            }
        }
    }
    if manifest.minify {
        for name in DEFAULT_MINIFYING_MODIFIERS {
            modifiers.push(Modifier::from_str(name)?);
        }
    }

    let (rules, mut fullmoon_visitors) = modifiers
        .into_iter()
        .fold((Vec::new(), Vec::new()), |(mut rules, mut fullmoon_visitors), modifier| {
            match modifier {
                Modifier::DarkluaRule(darklua_rule) => rules.push(darklua_rule),
                Modifier::FullMoonVisitor(fullmoon_visitor) => {
                    fullmoon_visitors.push(fullmoon_visitor);
                }
            }
            (rules, fullmoon_visitors)
        });

    let mut options = Options::new(input).with_configuration({
        // let mut config: Configuration = if bundle {
        //     toml::from_str("bundle = { require_mode = 'path' }").unwrap()
        // } else {
        //     Configuration::empty()
        // };
        let mut config = Configuration::empty();
        if bundle {
            config = config.with_bundle_configuration(
                BundleConfiguration::new(BundleRequireMode::default())
            );
        }
        config = config.with_generator(GeneratorParameters::RetainLines);

        rules.into_iter().fold(config, |config, rule| config.with_rule(rule))
    });
    options = options.with_output(&output);
    let result = darklua_core::process(&resources, options).map_err(|e| anyhow!(e))?;

    let success_count = result.success_count();
    let errors = result.collect_errors();
    let error_count = errors.len();
    if error_count > 0 {
        eprintln!(
            "{}{} error{} happened:",
            if success_count > 0 {
                "but "
            } else {
                ""
            },
            error_count,
            if error_count > 1 {
                "s"
            } else {
                ""
            }
        );

        errors.into_iter().for_each(|error| eprintln!("-> {}", error));

        return Err(anyhow!("darklua process was not successful"));
    }

    let mut created_files: Vec<PathBuf> = if output.is_dir() {
        let mut created_files = Vec::new();
        let mut entries = WalkDir::new(output);
        while let Some(entry) = entries.next().await {
            let path = entry?.path();
            if !matches!(path.extension().and_then(OsStr::to_str), Some("lua") | Some("luau")) {
                continue;
            }
            created_files.push(path);
        }
        created_files
    } else {
        vec![output.clone()]
    };

    let extension = manifest.file_extension();
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
            let mut ast = utils::parse_file(path, manifest.target_version()).await?;

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

pub async fn process(
    manifest: Manifest,
    additional_modifiers_a: Option<&mut Vec<Modifier>>
) -> Result<()> {
    let output_files = private_process(
        &manifest,
        &manifest.input(),
        &manifest.output(),
        additional_modifiers_a,
        false
    ).await?;
    let Some(polyfill) = manifest.polyfill() else {
        return Ok(());
    };
    let polyfill_cache = polyfill.cache().await?;
    let polyfill_config = polyfill_cache.config();

    // needed additional modifiers: inject_global_value
    let mut additional_modifiers = Vec::new();
    for (key, value) in polyfill_config {
        let value = if let Some(val) = polyfill.config().get(key) { val } else { value };
        let mut identifier = DALBIT_GLOBAL_IDENTIFIER_PREFIX.to_string();
        identifier.push_str(key);
        let inject_global_value = rules::InjectGlobalValue::boolean(identifier, *value);
        additional_modifiers.push(Modifier::DarkluaRule(Box::new(inject_global_value)));
    }

    if let Some(first_output) = output_files.first() {
        log::debug!("first output found!");
        let extension = if let Some(extension) = manifest.file_extension() {
            extension.to_owned()
        } else {
            first_output
                .extension()
                .ok_or_else(|| anyhow!("Failed to get extension from output file."))?
                .to_string_lossy()
                .into_owned()
        };
        if
            let Some(module_path) = first_output
                .parent()
                .map(|parent| { parent.join(polyfill.injection_path()).with_extension(extension) })
        {
            let _ = private_process(
                &manifest,
                polyfill_cache.globals_path(),
                &module_path,
                Some(&mut additional_modifiers),
                true
            ).await?;

            let mut exports = polyfill_cache.globals_exports().to_owned();
            for (key, value) in polyfill.globals() {
                if exports.contains(key) {
                    if !value {
                        exports.remove(key);
                    }
                } else {
                    return Err(anyhow!("Invalid global `{}`", key));
                }
            }

            log::info!("[injector] exports to be injected: {:?}", exports);

            let injector = Injector::new(
                module_path,
                exports,
                manifest.target_version().to_lua_version(),
                polyfill_cache.removes().to_owned()
            );

            for source_path in &output_files {
                injector.inject(source_path).await?;
            }
        }
    }
    Ok(())
}
