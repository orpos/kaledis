use std::str::FromStr;

use anyhow::{anyhow, Result};
use darklua_core::rules::Rule;
use full_moon::{ast::Ast, visitors::VisitorMut};

pub(crate) mod ast_util;
mod relative_path;
mod convert_bit32;
mod optimize_table_initializers;
mod remove_generalized_iteration;
mod remove_number_literals;
mod remove_redeclared_keys;
mod runtime_identifier;
mod get_love_modules;

pub use relative_path::*;
pub use get_love_modules::*;
pub use convert_bit32::*;
pub use optimize_table_initializers::*;
pub use remove_generalized_iteration::*;
pub use remove_number_literals::*;
pub use remove_redeclared_keys::*;
pub use runtime_identifier::*;

pub trait VisitorMutWrapper {
    fn visit_ast_boxed(&mut self, ast: Ast) -> Ast;
}

impl<T: VisitorMut> VisitorMutWrapper for T {
    fn visit_ast_boxed(&mut self, ast: Ast) -> Ast {
        self.visit_ast(ast)
    }
}

pub enum Modifier {
    DarkluaRule(Box<dyn Rule>),
    FullMoonVisitor(Box<dyn VisitorMutWrapper>),
}

impl FromStr for Modifier {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let modifier = match s {
            REMOVE_GENERALIZED_ITERATION_MODIFIER_NAME => {
                Modifier::DarkluaRule(Box::<RemoveGeneralizedIteration>::default())
            }
            REMOVE_NUMBER_LITERALS_MODIFIER_NAME => {
                Modifier::DarkluaRule(Box::<RemoveNumberLiterals>::default())
            }
            OPTIMIZE_TABLE_INITIALIZERS_MODIFIER_NAME => {
                Modifier::DarkluaRule(Box::<OptimizeTableInitializers>::default())
            }
            CONVERT_BIT32_MODIFIER_NAME => Modifier::FullMoonVisitor(Box::new(
                ConvertBit32::default(),
            )
                as Box<dyn VisitorMutWrapper>),
            REMOVE_REDECLARED_KEYS_RULE_NAME => {
                Modifier::DarkluaRule(Box::<RemoveRedeclaredKeys>::default())
            }
            _ => Modifier::DarkluaRule(s.parse::<Box<dyn Rule>>().map_err(|err| anyhow!(err))?),
        };

        Ok(modifier)
    }
}
