use darklua_core::nodes::{
    AssignStatement, Block, Expression, FunctionCall, FunctionExpression, Identifier,
    IndexExpression, LocalAssignStatement, ParentheseExpression, ReturnStatement, Statement,
    StringExpression, TableEntry, TableExpression,
};
use darklua_core::process::{DefaultVisitor, Evaluator, LuaValue, NodeProcessor, NodeVisitor};
use darklua_core::rules::{Context, RuleConfiguration, RuleConfigurationError, RuleProperties};

use super::runtime_identifier::RuntimeIdentifierBuilder;
use darklua_core::rules::{Rule, RuleProcessResult};

#[derive(Default)]
struct Processor {
    evaluator: Evaluator,
    table_identifier: String,
    skip_next_table_exp: bool,
}

impl Processor {
    fn skip(&mut self, active: bool) {
        self.skip_next_table_exp = active;
    }
}

use std::collections::HashMap;
use std::fmt::Debug;

impl NodeProcessor for Processor {
    fn process_expression(&mut self, exp: &mut Expression) {
        if let Expression::Table(table_exp) = exp {
            if self.skip_next_table_exp {
                self.skip(false);
                return;
            }
            let entries = table_exp.mutate_entries();
            let mut numeral_table = HashMap::new();
            let mut str_table = HashMap::new();
            let mut num_index: usize = 0;
            let mut side_effect_stmts: Vec<Statement> = Vec::new();

            for (i, entry) in entries.iter().enumerate() {
                match entry {
                    TableEntry::Index(index_entry) => {
                        let value = self.evaluator.evaluate(index_entry.get_key());
                        match value {
                            LuaValue::Number(lua_index) => {
                                if lua_index.fract() == 0.0 && lua_index > 0.0 {
                                    let key = (lua_index as usize) - 1;
                                    if side_effect_stmts.is_empty() {
                                        numeral_table.insert(key, i);
                                    } else {
                                        let assignment = AssignStatement::from_variable(
                                            IndexExpression::new(
                                                Identifier::new(self.table_identifier.as_str()),
                                                key + 1,
                                            ),
                                            index_entry.get_value().clone(),
                                        );
                                        side_effect_stmts.push(assignment.into());
                                    }
                                }
                            }
                            LuaValue::String(key) => {
                                if side_effect_stmts.is_empty() {
                                    str_table.insert(key, i);
                                } else {
                                    let assignment = AssignStatement::from_variable(
                                        IndexExpression::new(
                                            Identifier::new(self.table_identifier.as_str()),
                                            StringExpression::from_value(key),
                                        ),
                                        index_entry.get_value().clone(),
                                    );
                                    side_effect_stmts.push(assignment.into());
                                }
                            }
                            LuaValue::Unknown => {
                                let assignment = AssignStatement::from_variable(
                                    IndexExpression::new(
                                        Identifier::new(self.table_identifier.as_str()),
                                        index_entry.get_key().clone(),
                                    ),
                                    index_entry.get_value().clone(),
                                );
                                side_effect_stmts.push(assignment.into());
                            }
                            _ => (),
                        }
                    }
                    TableEntry::Value(_) => {
                        numeral_table.insert(num_index, i);
                        num_index += 1;
                    }
                    TableEntry::Field(field_entry) => {
                        let key = field_entry.get_field().get_name();
                        str_table.insert(key.to_owned(), i);
                    }
                }
            }

            let mut keys: Vec<_> = numeral_table.keys().collect();
            keys.sort();
            let mut new_entries: Vec<TableEntry> = Vec::new();

            for i in keys {
                let v = numeral_table[i];
                let entry = &entries[v];
                let new_entry = match entry {
                    TableEntry::Index(index_entry) => {
                        if *i <= num_index {
                            Some(TableEntry::Value(index_entry.get_value().clone()))
                        } else {
                            Some(TableEntry::Index(index_entry.clone()))
                        }
                    }
                    TableEntry::Value(exp) => Some(TableEntry::Value(exp.clone())),
                    _ => None,
                };
                if let Some(new_entry) = new_entry {
                    new_entries.push(new_entry);
                }
            }

            for (_, v) in str_table {
                let entry = &entries[v];
                new_entries.push(entry.clone());
            }

            entries.clear();
            for ent in new_entries {
                entries.push(ent);
            }

            if !side_effect_stmts.is_empty() {
                let var = Identifier::new(self.table_identifier.as_str());
                let table_stmt = TableExpression::new(entries.clone());
                self.skip(true);
                let local_assign_stmt =
                    LocalAssignStatement::new(vec![var.clone().into()], vec![table_stmt.into()]);
                side_effect_stmts.insert(0, local_assign_stmt.into());
                let return_stmt = ReturnStatement::one(var);
                let func_block = Block::new(side_effect_stmts, Some(return_stmt.into()));
                let func = Expression::Function(FunctionExpression::from_block(func_block));
                let parenthese_func = ParentheseExpression::new(func);
                let func_call = FunctionCall::from_prefix(parenthese_func);
                let call_exp = Expression::Call(Box::new(func_call));
                *exp = call_exp;
            }
        }
    }
}

pub const REMOVE_REDECLARED_KEYS_RULE_NAME: &str = "remove_redeclared_keys";

/// A rule that removes redeclared keys in table and organize the components of a mixed table
#[derive(Debug, PartialEq, Eq)]
pub struct RemoveRedeclaredKeys {
    runtime_identifier_format: String,
}

impl Default for RemoveRedeclaredKeys {
    fn default() -> Self {
        Self {
            runtime_identifier_format: "_DARKLUA_REMOVE_REDECLARED_KEYS_{name}{hash}".to_string(),
        }
    }
}

impl Rule for RemoveRedeclaredKeys {
    fn process(&self, block: &mut Block, _: &Context) -> RuleProcessResult {
        let var_builder = RuntimeIdentifierBuilder::new(
            self.runtime_identifier_format.as_str(),
            format!("{block:?}").as_bytes(),
            None,
        )?;
        let mut processor = Processor {
            evaluator: Evaluator::default(),
            table_identifier: var_builder.build("tbl")?,
            skip_next_table_exp: false,
        };
        DefaultVisitor::visit_block(block, &mut processor);
        Ok(())
    }
}

impl RuleConfiguration for RemoveRedeclaredKeys {
    fn configure(&mut self, _: RuleProperties) -> Result<(), RuleConfigurationError> {
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        REMOVE_REDECLARED_KEYS_RULE_NAME
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        RuleProperties::new()
    }
}
