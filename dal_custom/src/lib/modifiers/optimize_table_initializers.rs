use std::str::FromStr;

use anyhow::anyhow;
use darklua_core::{
    nodes::{Arguments, Block, Expression, Prefix, TableExpression},
    process::{DefaultVisitor, NodeProcessor, NodeVisitor},
    rules::{Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties},
};

pub const OPTIMIZE_TABLE_INITIALIZERS_MODIFIER_NAME: &str = "optimize_table_initializers";

const DEFAULT_TABLE_LIBRARY: &str = "table";

#[non_exhaustive]
enum OptimizableTableMethod {
    Create,
    Freeze,
}

impl OptimizableTableMethod {
    fn try_optimize(&self, arguments: &Arguments) -> Option<Expression> {
        match self {
            OptimizableTableMethod::Create => {
                if let Arguments::Tuple(tuple) = arguments {
                    if tuple.len() < 2 {
                        return Some(Expression::Table(TableExpression::default()));
                    }
                }
            }
            OptimizableTableMethod::Freeze => match arguments {
                Arguments::Tuple(tuple) => {
                    let first_arg = tuple.iter_values().next();
                    return first_arg.cloned();
                }
                Arguments::Table(table) => {
                    return Some(Expression::Table(table.to_owned()));
                }
                _ => {
                    return None;
                }
            },
        };
        None
    }
}

impl FromStr for OptimizableTableMethod {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let optimizer = match s {
            "create" => OptimizableTableMethod::Create,
            "freeze" => OptimizableTableMethod::Freeze,
            _ => {
                return Err(anyhow!("Invalid OptimizableTableMethod `{}`", s));
            }
        };

        Ok(optimizer)
    }
}

struct Processor {}

impl NodeProcessor for Processor {
    fn process_expression(&mut self, exp: &mut Expression) {
        if let Expression::Call(func_call) = exp {
            let lib_and_call: Option<(&str, &str)> = match func_call.get_prefix() {
                Prefix::Field(field) => {
                    if let Prefix::Identifier(identifier) = field.get_prefix() {
                        Some((identifier.get_name(), field.get_field().get_name()))
                    } else {
                        None
                    }
                }
                Prefix::Index(index) => {
                    if let Expression::String(string) = index.get_index() {
                        if let Prefix::Identifier(identifier) = index.get_prefix() {
                            Some((identifier.get_name(), string.get_value()))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if let Some((lib_name, call_name)) = lib_and_call {
                if lib_name != DEFAULT_TABLE_LIBRARY {
                    return;
                }
                if let Ok(method) = OptimizableTableMethod::from_str(call_name) {
                    let new_exp = method.try_optimize(func_call.get_arguments());
                    if let Some(new_exp) = new_exp {
                        *exp = new_exp;
                    }
                }
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct OptimizeTableInitializers {}

impl FlawlessRule for OptimizeTableInitializers {
    fn flawless_process(&self, block: &mut Block, _: &Context) {
        let mut processor = Processor {};
        DefaultVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for OptimizeTableInitializers {
    fn configure(&mut self, _: RuleProperties) -> Result<(), RuleConfigurationError> {
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        OPTIMIZE_TABLE_INITIALIZERS_MODIFIER_NAME
    }

    fn serialize_to_properties(&self) -> darklua_core::rules::RuleProperties {
        RuleProperties::new()
    }
}
