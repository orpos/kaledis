// This is a custom rule needed for making relative imports like ../ work in love2d   (letiul asked for this btw)
// love2d gets with base the root of the project so i just translate it

use std::path::{self, Path, PathBuf};
use colored::Colorize;

use darklua_core::{
    nodes::{Arguments, Block, Expression, Prefix, StringExpression},
    process::{DefaultVisitor, NodeProcessor, NodeVisitor},
    rules::{
        Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleMetadata,
        RuleProperties,
    },
};
use fs_err::canonicalize;

pub const RELATIVE_PATH_MODIFIER_NAME: &str = "path_modifier";

struct Processor<'a> {
    path: &'a Path,
    paths: &'a [(String, String)],
    project_root_src: &'a PathBuf,
    project_root: &'a PathBuf,
}

// This exists to stop conflicts with folders with a dot
fn to_module_path<T: Into<PathBuf>>(
    caller_path: &PathBuf,
    project_root_src: &PathBuf,
    project_root: &PathBuf,
    require_pathbuf: T,
) -> String {
    let require_path = require_pathbuf.into();
    let stripped_caller = caller_path
        .strip_prefix(project_root)
        .unwrap_or(caller_path);

    // first we try it normally, then we try with luau and at last lua
    let mut path_buf: PathBuf = canonicalize(&require_path)
        .or(canonicalize(require_path.with_extension("luau")))
        .or(canonicalize(require_path.with_extension("lua")))
        .or(canonicalize(require_path.with_file_name("init.luau")))
        .or(canonicalize(require_path.with_file_name("init.lua")))
        .unwrap_or_default()
        .with_extension("");

    path_buf = PathBuf::from(
        path_buf
            .to_string_lossy()
            // when you canonicalize in windows it leaves this:
            .trim_start_matches("\\\\?\\"),
    );

    path_buf
        .strip_prefix(project_root_src)
        .unwrap_or_else(|_| {
            path_buf.strip_prefix(project_root).unwrap_or_else(|_| {
                let stripped_file = require_path
                    .strip_prefix(project_root)
                    .unwrap_or(&require_path);
                println!(
                    "{} Unable to resolve require path : {:?};\n when resolving in {}",
                    "[!]".yellow(),
                    &stripped_file,
                    stripped_caller.to_string_lossy().red()
                );
                &require_path
            })
        })
        .iter()
        // due to love2d custom require we use this to stop conflicts
        // as such this should be enforced in build time and by this rule
        .map(|x| x.to_str().unwrap().replace(".", "__"))
        .collect::<Vec<String>>()
        .join(".")
        .trim_end_matches(".luau")
        .to_string()
}

pub fn find_init_luau_folder(
    project_root: &PathBuf,
    start_dir: impl AsRef<Path>,
) -> Option<PathBuf> {
    let mut current = start_dir.as_ref().to_path_buf();

    loop {
        let candidate = current.join("init.luau");

        if std::fs::metadata(&candidate).is_ok() {
            return Some(current);
        }

        if &current == project_root || !current.pop() {
            // reached filesystem root
            return None;
        }
    }
}

impl<'a> NodeProcessor for Processor<'a> {
    fn process_function_call(&mut self, function_call: &mut darklua_core::nodes::FunctionCall) {
        if let Prefix::Identifier(identifier) = function_call.get_prefix()
            && identifier.get_name() == "require"
            && let Arguments::Tuple(dat) = function_call.mutate_arguments()
            && let Some(Expression::String(expr)) = dat.iter_mut_values().next()
        {
            let caller = self.path;

            let require = expr.get_value();
            let require_str = String::from_utf8_lossy(require).to_string();

            let mut aliases = self.paths.to_vec();
            if require.starts_with(b"@self") {
                let init_path = find_init_luau_folder(self.project_root, caller);
                if let Some(path) = init_path {
                    if let Ok(relative_path) = path.strip_prefix(self.project_root) {
                        aliases.push((
                            "self".to_string(),
                            relative_path.to_string_lossy().to_string(),
                        ));
                    } else {
                        eprintln!(
                            "{} @self resolved outside of project root \n when working in {:?}",
                            "[!]".yellow(),
                            self.path
                        );
                    }
                } else {
                    eprintln!(
                        "{} Unable to resolve @self \n when working in {:?}",
                        "[!]".yellow(),
                        self.path
                    );
                }
            }
            if let Some(alias) = aliases
                .iter()
                .find(|x| require.starts_with(&format!("@{}", x.0).as_bytes()))
            {
                let resolved_alias = self.project_root.join(&alias.1);
                let relative_require = require_str.strip_prefix(&format!("@{}", alias.0)).unwrap().trim_start_matches('/');

                let pth = resolved_alias.join(PathBuf::from(relative_require));
                *expr = StringExpression::from_value(to_module_path(
                    &self.path.to_path_buf(),
                    self.project_root_src,
                    self.project_root,
                    pth,
                ));
                return;
            };

            // if we got here, no alias was triggered
            if require.starts_with(b"../") || require.starts_with(b"./") {
                let caller_dir = self.path.parent().unwrap();
                let relative_base = if self.path.file_name().is_some_and(|name| name == "init.luau")
                {
                    caller_dir.parent().unwrap_or(caller_dir)
                } else {
                    caller_dir
                };
                let pth = path::absolute(
                    relative_base
                        .join(String::from_utf8_lossy(require).to_string()),
                )
                .expect("Failed To Find Module");

                *expr = StringExpression::from_value(to_module_path(
                    &self.path.to_path_buf(),
                    self.project_root_src,
                    self.project_root,
                    pth,
                ));

                return;
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct ModifyPathModifier {
    pub metadata: darklua_core::rules::RuleMetadata,
    pub project_root_src: PathBuf,
    pub project_root: PathBuf,
    pub paths: Vec<(String, String)>,
}

impl FlawlessRule for ModifyPathModifier {
    fn flawless_process(&self, block: &mut Block, ctx: &Context) {
        let mut processor = Processor {
            path: ctx.current_path(),
            project_root_src: &self.project_root_src,
            project_root: &self.project_root,
            paths: self.paths.as_slice(),
        };
        DefaultVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for ModifyPathModifier {
    fn configure(&mut self, _: RuleProperties) -> Result<(), RuleConfigurationError> {
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        RELATIVE_PATH_MODIFIER_NAME
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        RuleProperties::new()
    }

    fn has_properties(&self) -> bool {
        false
    }

    fn set_metadata(&mut self, metadata: RuleMetadata) {
        self.metadata = metadata;
    }

    fn metadata(&self) -> &darklua_core::rules::RuleMetadata {
        &self.metadata
    }
}
