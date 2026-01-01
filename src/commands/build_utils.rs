use std::{fs, path::PathBuf, str::FromStr};

use crate::dalbit::manifest::Manifest;
use serde_json::{Value, from_str};
use tokio::fs::{read_to_string, try_exists};

use crate::toml_conf::{Config, CustomPolyfillConfig};

#[derive(Clone)]
pub struct Paths {
    pub root: PathBuf,
    pub build : PathBuf,
    pub dist : PathBuf,
    pub src: PathBuf,
    pub assets : Option<PathBuf>,
    pub polyfill_path: Option<PathBuf>
}

pub fn normalize_lua_path(path: &PathBuf, root: &PathBuf, alternative: &PathBuf) -> PathBuf {
    let mut new_path = PathBuf::new();
    let target = path
        .strip_prefix(&root)
        .unwrap_or_else(|_| path.strip_prefix(alternative).unwrap());
    if let Some(parent) = target.parent() {
        let mut comps = parent.iter();

        // Push the first path component untouched
        if let Some(first) = comps.next() {
            new_path.push(first);
        }

        // Replace dots in the remaining directories
        for comp in comps {
            let comp_str = comp.to_string_lossy().replace('.', "__");
            new_path.push(comp_str);
        }
    }

    // Append the filename without changes
    if let Some(file_name) = path.file_name() {
        new_path.push(file_name);
    }

    root.join(new_path)
}

pub async fn get_transpiler(
    one_file: bool,
    polyfill_config: Option<&CustomPolyfillConfig>,
) -> anyhow::Result<Manifest> {
    let mut manifest = Manifest {
        minify: true,
        bundle: one_file,
        ..Default::default()
    };
    if let Some(polyfill) = polyfill_config {
        manifest.polyfill = Some(polyfill.polyfill().await.unwrap());
    }
    macro_rules! add_modifiers {
        ($modifier:expr) => {
            manifest.modifiers.insert($modifier.to_string(), true);
        };
        ($modifier:expr, $($modi:expr),+) => {
            add_modifiers!($modifier);
            add_modifiers!($($modi), +);
        };
    }
    add_modifiers!(
        // "rename_variables",
        "remove_empty_do",
        "remove_spaces",
        "remove_unused_while",
        "remove_unused_variable",
        "remove_unused_if_branch"
    );
    // Thanks to new dalbit version this was made much easier
    if let Some(polyfill) = manifest.polyfill.as_ref() {
        if polyfill_config.is_none() {
            polyfill.cache()?;
            
        }
    }
    return Ok(manifest);
}

impl Paths {
    pub fn from_root(root: PathBuf, value: Config) -> Self {
        Self {
            build: root.join(".build"),
            src: root.join(PathBuf::from(value.project.src_path.unwrap_or(root.to_string_lossy().to_string()))),
            polyfill_path: value
                .polyfill
                .as_ref()
                .map(|x| x.location.as_ref().map(|x| root.join(&x)))
                .flatten(),
            assets: value.project.asset_path.map(|x|PathBuf::from_str(&x).unwrap()),
            dist: root.join("dist"),
            root,
        }
    }
}
pub fn uppercase_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

pub async fn read_aliases(path: &PathBuf) -> anyhow::Result<Vec<(String, String)>> {
    let mut buffer = Vec::new();
    if !try_exists(path.join(".luaurc")).await? {
        return Ok(vec![])
    }
    
    let contents = read_to_string(path.join(".luaurc")).await?;
    let json : Value = from_str(&contents)?;
    if let Some(Value::Object(aliases)) = json.get("aliases") {
        for (key, value) in aliases.iter() {
            if let Some(value_str) = value.as_str() {
                buffer.push((key.to_owned(), value_str.to_owned()));
            }
        }
    }

    Ok(buffer)
}