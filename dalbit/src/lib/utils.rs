use std::{collections::HashSet, path::PathBuf};

use anyhow::{anyhow, Result};
use full_moon::{
    ast::{Ast, Expression, Field, LastStmt},
    tokenizer::TokenKind,
};
use tokio::fs;

use crate::TargetVersion;

pub enum ParseTarget {
    FullMoonAst(Ast),
    File(PathBuf, TargetVersion),
}

pub(crate) async fn parse_file(path: &PathBuf, target_version: &TargetVersion) -> Result<Ast> {
    let code = fs::read_to_string(path).await?;
    let ast = full_moon::parse_fallible(code.as_str(), target_version.to_lua_version().clone())
        .into_result()
        .map_err(|errors| anyhow!("full_moon parsing error: {:?}", errors))?;

    Ok(ast)
}

/// Gets exports of lua modules by parsing last statement's table constructor.
pub async fn get_exports_from_last_stmt(target: &ParseTarget) -> Result<Option<HashSet<String>>> {
    let ast = match target {
        ParseTarget::FullMoonAst(ast) => ast,
        ParseTarget::File(path, target_version) => &parse_file(path, target_version).await?,
    };
    let block = ast.nodes();

    if let Some(exports) = block
        .last_stmt()
        .and_then(|last_stmt| match last_stmt {
            LastStmt::Return(return_stmt) => return_stmt.returns().first(),
            _ => None,
        })
        .and_then(|first_return| match first_return.value() {
            Expression::TableConstructor(table_constructor) => Some(table_constructor),
            _ => None,
        })
        .map(|table_constructor| {
            let mut exports: HashSet<String> = HashSet::new();
            for field in table_constructor.fields() {
                if let Some(new_export) = match field {
                    Field::ExpressionKey {
                        brackets: _,
                        key,
                        equal: _,
                        value: _,
                    } => {
                        if let Expression::String(string_token) = key {
                            let string_token = string_token.token();
                            if let TokenKind::StringLiteral = string_token.token_kind() {
                                log::debug!("[get_exports_from_last_stmt] ExpressionKey token kind: {:?} string: {}", string_token.token_kind(), string_token.to_string());
                                Some(string_token.to_string().trim().to_owned())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    Field::NameKey {
                        key,
                        equal: _,
                        value: _,
                    } => {
                        let key = key.token();
                        if let TokenKind::Identifier = key.token_kind() {
                            log::debug!("[get_exports_from_last_stmt] NameKey token kind: {:?} string: {}", key.token_kind(), key.to_string());
                            Some(key.to_string().trim().to_owned())
                        } else {
                            None
                        }
                    }
                    _ => None,
                } {
                    exports.insert(new_export);
                }
            }
            exports
        })
    {
        return Ok(Some(exports));
    }

    Ok(None)
}
