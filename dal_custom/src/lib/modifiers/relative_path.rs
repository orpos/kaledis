// This is a custom rule needed for making relative imports like ../
// in love2d   (lettuce asked for this btw)

use std::path::{ self, Path, PathBuf } ;

use darklua_core::{
    nodes::{ Arguments, Block, Expression, Prefix, StringExpression },
    process::{ DefaultVisitor, NodeProcessor, NodeVisitor },
    rules::{ Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties },
};

pub const RELATIVE_PATH_MODIFIER_NAME: &str = "relative_path";

struct Processor<'a> {
    path: &'a Path,
    project_root: &'a PathBuf,
}

impl<'a> NodeProcessor for Processor<'a> {
    fn process_function_call(&mut self, function_call: &mut darklua_core::nodes::FunctionCall) {
        if let Prefix::Identifier(a) = function_call.get_prefix() {
            if a.get_name() == "require" {
                let args = function_call.mutate_arguments();
                if let Arguments::Tuple(dat) = args {
                    if let Some(Expression::String(expr)) = dat.iter_mut_values().next() {
                        if expr.get_value().starts_with("../") {
                            let pth = path
                                ::absolute(self.path.parent().unwrap().join(expr.get_value()))
                                .expect("A");
                            let ceb = pth.strip_prefix(self.project_root).expect("Path strip.");


                            *expr = StringExpression::from_value(
                                &ceb
                                    .to_path_buf()
                                    .into_iter()
                                    .map(|x| x.to_str().unwrap())
                                    .collect::<Vec<&str>>()
                                    .join(".")
                            );
                        }
                    }
                }
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct ModifyRelativePath {
    pub project_root: PathBuf,
}

impl FlawlessRule for ModifyRelativePath {
    fn flawless_process(&self, block: &mut Block, ctx: &Context) {
        let mut processor = Processor {
            path: ctx.current_path(),
            project_root: &self.project_root,
        };
        DefaultVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for ModifyRelativePath {
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