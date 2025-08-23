// This is a custom rule needed for making relative imports like ../ work in love2d   (letiul asked for this btw)
// love2d gets with base the root of the project so i just translate it

use std::path::{ self, Path, PathBuf };

use darklua_core::{
    nodes::{ Arguments, Block, Expression, Prefix, StringExpression },
    process::{ DefaultVisitor, NodeProcessor, NodeVisitor },
    rules::{ Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties },
};

pub const RELATIVE_PATH_MODIFIER_NAME: &str = "path_modifier";

struct Processor<'a> {
    path: &'a Path,
    paths: &'a [(String, String)],
    project_root_src: &'a PathBuf,
    project_root: &'a PathBuf,
}

// This exists to stop conflicts with folders with a dot
fn to_module_path<T: Into<PathBuf>>(
    project_root_src: &PathBuf,
    project_root: &PathBuf,
    new_path: T
) -> String {
    let path_buf: PathBuf = new_path.into();
    return path_buf
        .strip_prefix(project_root_src)
        .unwrap_or_else(|_| path_buf.strip_prefix(project_root).unwrap())
        .into_iter()
        // due to love2d custom require we use this to stop conflicts
        // as such this should be enforced in build time and by this rule
        .map(|x| x.to_str().unwrap().replace(".", "__"))
        .collect::<Vec<String>>()
        .join(".")
        .trim_end_matches(".luau")
        .to_string();
}

pub fn find_init_luau_folder(
    project_root: &PathBuf,
    start_dir: impl AsRef<Path>
) -> std::io::Result<Option<PathBuf>> {
    let mut current = start_dir.as_ref().to_path_buf();

    loop {
        let candidate = current.join("init.luau");

        if std::fs::metadata(&candidate).is_ok() {
            return Ok(Some(current));
        }

        if &current == project_root || !current.pop() {
            // reached filesystem root
            return Ok(None);
        }
    }
}

impl<'a> NodeProcessor for Processor<'a> {
    fn process_function_call(&mut self, function_call: &mut darklua_core::nodes::FunctionCall) {
        if let Prefix::Identifier(identifier) = function_call.get_prefix() {
            if identifier.get_name() == "require" {
                let args = function_call.mutate_arguments();
                if let Arguments::Tuple(dat) = args {
                    if let Some(Expression::String(expr)) = dat.iter_mut_values().next() {
                        let require = expr.get_value().to_string();
                        let is_relative = require.starts_with("../") || require.starts_with("./");
                        for preset in self.paths {
                            if
                                let Some(requested_package) = require.strip_prefix(
                                    &format!("@{}/", preset.0)
                                )
                            {
                                let pth = path
                                    ::absolute(
                                        self.project_root
                                            .join(&preset.1)
                                            .join(PathBuf::from(requested_package))
                                    )
                                    .expect("Failed To Find Module");
                                *expr = StringExpression::from_value(
                                    to_module_path(self.project_root_src, self.project_root, pth)
                                );
                                return;
                            };
                        }
                        if is_relative || require.starts_with("@self") {
                            let pth: PathBuf;
                            if let Some(data) = require.strip_prefix("@self") {
                                let init_folder = find_init_luau_folder(
                                    self.project_root,
                                    self.path
                                ).unwrap();
                                let module_path = init_folder
                                    .as_ref()
                                    .unwrap_or(self.project_root)
                                    .join(
                                        if data.starts_with("/") {
                                            data.strip_prefix("/").unwrap()
                                        } else {
                                            "init"
                                        }
                                    );
                                pth = path::absolute(module_path).expect("Failed To Find Module");
                            } else {
                                pth = path
                                    ::absolute(
                                        self.path
                                            .parent()
                                            .unwrap()
                                            .join(&require.trim_start_matches("@self/"))
                                    )
                                    .expect("Failed To Find Module");
                            }

                            *expr = StringExpression::from_value(
                                to_module_path(self.project_root_src, self.project_root, pth)
                            );
                        }
                    }
                }
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct ModifyPathModifier {
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
}
