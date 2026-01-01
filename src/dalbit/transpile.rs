use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
};

use super::modifiers::VisitorMutWrapper;
use anyhow::anyhow;
use darklua_core::{
    BundleConfiguration, Configuration, GeneratorParameters, Options, Resources,
    rules::{self, Rule, bundle::BundleRequireMode},
};
use fs_err::remove_file;
use full_moon::{
    tokenizer::{Token, TokenType},
    visitors::Visitor,
};
use indexmap::{IndexMap, IndexSet};
use rayon::{
    ThreadPoolBuilder,
    iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator},
    vec,
};
use walkdir::WalkDir;

use crate::{
    commands::build_utils::{Paths, normalize_lua_path},
    dalbit::{
        manifest::Manifest,
        modifiers::{GetLoveModules, Modifier, ModifyPathModifier},
        polyfill::{Polyfill, PolyfillCache, PolyfillCacheInfo},
        utils,
    },
};

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

#[derive(Debug)]
struct CollectUsedLibraries {
    libraries: HashSet<String>,
    love_libraries: HashSet<String>,
    used_libraries: HashSet<String>,
    love_used_libraries: HashSet<String>,
}

impl CollectUsedLibraries {
    fn new(libraries: HashSet<String>, love_libraries: HashSet<String>) -> Self {
        Self {
            libraries,
            love_libraries,
            used_libraries: HashSet::new(),
            love_used_libraries: HashSet::new(),
        }
    }
}

impl Visitor for CollectUsedLibraries {
    fn visit_identifier(&mut self, identifier: &Token) {
        if let TokenType::Identifier { identifier } = identifier.token_type() {
            let identifier = identifier.to_string();
            if self.love_libraries.contains(&identifier) {
                self.love_used_libraries.insert(identifier);
            } else if self.libraries.contains(&identifier) {
                self.used_libraries.insert(identifier);
            }
        }
    }
}

pub fn clean_polyfill() {
    let polyfill_output = dirs::cache_dir()
        .expect("Unable to find Cache directory")
        .join("__polyfill_kaledis__.lua");
    if polyfill_output.exists() {
        remove_file(polyfill_output).unwrap();
    }
}

// We separated this from the main process to make it more performant and to generate the polyfills only once
pub fn process_polyfill(polyfill: &Polyfill) -> anyhow::Result<InjectPolyfill> {
    let polyfill_cache = polyfill.cache()?;
    let config = polyfill_cache.config();

    let mut additional_modifiers = Vec::new();
    for (key, value) in config {
        let value = if let Some(val) = polyfill.config.get(key) {
            val
        } else {
            value
        };
        let mut identifier = DALBIT_GLOBAL_IDENTIFIER_PREFIX.to_string();
        identifier.push_str(key);
        let inject_global_value = rules::InjectGlobalValue::boolean(identifier, *value);
        additional_modifiers.push(Modifier::DarkluaRule(Box::new(inject_global_value)));
    }

    let polyfill_output = dirs::cache_dir()
        .expect("Unable to find Cache directory")
        .join("__polyfill_kaledis__.lua");
    if polyfill_output.exists() {
        return Ok(polyfill_cache.into());
    }

    private_process(
        &Manifest {
            bundle: true,
            hmr: false,
            minify: true,
            polyfill: None,
            ..Default::default()
        },
        polyfill_cache.globals_path(),
        &polyfill_output,
        true,
        None,
        &vec![],
        None,
        None,
        None,
    )
    .unwrap();
    Ok(polyfill_cache.into())
}

pub fn get_polyfill_contents() -> Option<String> {
    let polyfill_output = dirs::cache_dir()
        .expect("Unable to find Cache directory")
        .join("__polyfill_kaledis__.lua");
    if polyfill_output.exists() {
        return std::fs::read_to_string(polyfill_output).ok();
    }
    None
}

#[derive(Clone, Debug)]
pub struct InjectPolyfill {
    pub path: String,
    pub exported: HashSet<String>,
    pub removes: Vec<String>,
}
impl From<PolyfillCache> for InjectPolyfill {
    fn from(value: PolyfillCache) -> Self {
        InjectPolyfill {
            path: "__polyfill__".to_string(),
            exported: value.globals_exports().clone(),
            removes: value.removes().clone().unwrap_or(vec![]),
        }
    }
}

// This is heavily customized to suffice the needs of kaledis in regards of performance
// The reason we transformed this to a sync process
// is to support multi threading
fn private_process(
    manifest: &Manifest,
    input: &PathBuf,
    output: &PathBuf,
    bundle: bool,
    paths: Option<&Paths>,
    aliases: &[(String, String)],
    used_modules: Option<Arc<Mutex<IndexSet<String>>>>,
    additional_modifiers: Option<Vec<Modifier>>,
    polyfill: Option<InjectPolyfill>,
) -> anyhow::Result<Vec<PathBuf>> {
    let resources = Resources::from_file_system();

    let mut modifiers = Vec::new();
    if let Some(mut new_modifiers) = additional_modifiers {
        modifiers.append(&mut new_modifiers);
    }

    // Love specific rules
    if let Some(paths) = paths {
        modifiers.push(Modifier::DarkluaRule(Box::new(ModifyPathModifier {
            paths: aliases.to_vec(),
            project_root: paths.root.clone(),
            project_root_src: paths.src.clone(),
        })));
    }

    if let Some(modules) = &used_modules {
        modifiers.push(Modifier::DarkluaRule(Box::new(GetLoveModules {
            modules: Arc::clone(modules),
        })));
    }

    {
        let mut transpiling_modifiers = IndexMap::new();
        for name in DEFAULT_LUAU_TO_LUA_MODIFIERS {
            transpiling_modifiers.insert(name, true);
        }
        for (name, enabled) in manifest.modifiers() {
            let name = name.as_str();
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

    let (rules, mut fullmoon_visitors) = modifiers.into_iter().fold(
        (Vec::new(), Vec::new()),
        |(mut rules, mut fullmoon_visitors), modifier| {
            match modifier {
                Modifier::DarkluaRule(darklua_rule) => rules.push(darklua_rule),
                Modifier::FullMoonVisitor(fullmoon_visitor) => {
                    fullmoon_visitors.push(fullmoon_visitor);
                }
            }
            (rules, fullmoon_visitors)
        },
    );

    let mut options = Options::new(input).with_configuration({
        let mut config = Configuration::empty();
        if bundle {
            config = config
                .with_bundle_configuration(BundleConfiguration::new(BundleRequireMode::default()));
        }
        config = config.with_generator(GeneratorParameters::RetainLines);

        rules
            .into_iter()
            .fold(config, |config, rule| config.with_rule(rule))
    });
    options = options.with_output(&output);
    let result = darklua_core::process(&resources, options).map_err(|e| anyhow!(e))?;

    let success_count = result.success_count();
    let errors = result.collect_errors();
    let error_count = errors.len();
    if error_count > 0 {
        eprintln!(
            "{}{} error{} happened:",
            if success_count > 0 { "but " } else { "" },
            error_count,
            if error_count > 1 { "s" } else { "" }
        );

        errors
            .into_iter()
            .for_each(|error| eprintln!("-> {}", error));

        return Err(anyhow!("darklua process was not successful"));
    }

    let mut created_files: Vec<PathBuf> = if output.is_dir() {
        let mut created_files = Vec::new();
        for entry in WalkDir::new(output).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            if !matches!(
                path.extension().and_then(OsStr::to_str),
                Some("lua") | Some("luau")
            ) {
                continue;
            }
            created_files.push(path.into());
        }
        created_files
    } else {
        vec![output.clone()]
    };
    for path in &mut created_files {
        let mut ast = utils::parse_file(path, false)?;

        for visitor in &mut fullmoon_visitors {
            ast = visitor.visit_ast_boxed(ast);
        }

        let ast_text = ast.to_string();
        let mut start_lines = vec![];

        if manifest.hmr {
            start_lines.push("local lick=require(\"lick\")".to_string());
        }

        // Here we inject the libraries that are polyfilled
        let love2d_libraries = vec!["Socket", "Enet", "http", "ftp", "smtp", "mime", "ltn12"];
        let internal_socket_modules = vec![
            "dns", "tcp", "udp", "unix", "connect", "bind", "select", "sleep", "gettime",
            "protect", "newtry", "sink", "source", "skip", "choose",
        ];
        let mut used_libraries = CollectUsedLibraries::new(
            polyfill
                .as_ref()
                .map(|x| x.exported.clone())
                .unwrap_or(HashSet::new()),
            HashSet::from_iter(love2d_libraries.iter().map(|x| x.to_string())),
        );
        used_libraries.visit_ast(&ast);

        let is_using_socket = used_libraries
            .love_used_libraries
            .get(&"Socket".to_string())
            .is_some();
        let is_using_enet = used_libraries
            .love_used_libraries
            .get(&"Enet".to_string())
            .is_some();

        if is_using_enet {
            start_lines.push("local Enet = require(\"enet\")".to_string());
        }
        if is_using_socket {
            let mut end: String = r#"local ___SOCKET = require("socket");
local Socket={socket=___SOCKET,"#
                .to_string();
            for internal in internal_socket_modules {
                end += &format!("{}=___SOCKET.{},", internal, internal);
            }
            for library in used_libraries.love_used_libraries {
                if library == "Enet" || library == "Socket" {
                    continue;
                }
                if library == "mime" || library == "ltn12" {
                    end += &format!("{}=require(\"{}\"),", library, library);
                    continue;
                }
                end += &format!("{}=require(\"socket.{}\"),", library, library);
            }
            end = end.trim_end_matches(",").to_string();
            end += "}";
            start_lines.push(end);
        }

        for lib in used_libraries.used_libraries {
            start_lines.push(format!(
                // Since we are in love2d and the requires are based on the root
                // we can do this, in any other situation we should not
                "local {}=require'{}'.{} ",
                // We use unwrap because, if you somehow put the used libraries, you
                // are already injecting the polyfill
                lib,
                polyfill.as_ref().map(|x| x.path.clone()).unwrap(),
                lib
            ));
        }

        if let Some(removes) = polyfill.as_ref().map(|x| &x.removes) {
            for lib in removes {
                // TODO: move this from here to the custom polyfill
                if lib == "io" || lib == "package" {
                    continue;
                }
                start_lines.push(format!("local {}=nil ", lib));
            }
        }

        let new_content = start_lines.join("\n") + &ast_text;

        let old_path = path.clone();
        path.set_extension("lua");
        let new_path = path.to_owned();
        if new_path != old_path && old_path.exists() {
            std::fs::remove_file(old_path)?;
        }

        std::fs::write(path, new_content)?;
    }

    Ok(created_files)
}

pub fn process_files(
    manifest: &Manifest,
    files: HashMap<PathBuf, PathBuf>,
    paths: Paths,
    aliases: &[(String, String)],
) -> anyhow::Result<Vec<String>> {
    let used_modules = Arc::new(Mutex::new(IndexSet::new()));
    let cache = manifest
        .polyfill
        .as_ref()
        .map(|polyfill| process_polyfill(polyfill).unwrap());

    let pool = ThreadPoolBuilder::new()
        .stack_size(4 * 1024 * 1024 * 1024)
        .build()
        .expect("Failed to build pool");
    pool.install(|| {
        files.into_par_iter().for_each(|(input, output)| {
            private_process(
                manifest,
                &input,
                &normalize_lua_path(&output, &paths.build, &paths.src),
                false,
                Some(&paths),
                aliases,
                Some(Arc::clone(&used_modules)),
                None,
                cache.clone(),
            )
            .unwrap();
        });
    });
    let value: Vec<String> = used_modules.lock().unwrap().iter().cloned().collect();
    Ok(value)
}
