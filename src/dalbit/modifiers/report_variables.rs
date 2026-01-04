use std::path::PathBuf;
// This is still in progress

use darklua_core::nodes::Statement;
use full_moon::{
    ast::{
        Call, Expression, FunctionArgs, FunctionCall, Prefix, Stmt, Suffix, Var,
        punctuated::{Pair, Punctuated},
        span::ContainedSpan,
    },
    node::Node,
    tokenizer::TokenReference,
    visitors::VisitorMut,
};

#[derive(Debug)]
pub struct ReportVariables {
    file_path: PathBuf,
}

impl VisitorMut for ReportVariables {
    fn visit_block(&mut self, node: full_moon::ast::Block) -> full_moon::ast::Block {
        let mut new_statements = vec![];

        for (statement, b) in node.stmts_with_semicolon().into_iter().cloned() {
            new_statements.push((statement.clone(), b));
            if let Stmt::Assignment(asm) = statement {
                for var in asm.variables() {
                    if let Var::Expression(a) = var {
                        // a.end_position()
                    }
                    if let Var::Name(x) = var {
                        let pos = x.end_position().unwrap();

                        // Here we fabricate the function call like: ___KALEDIS_REPORT_VARIABLE(
                        let name_token =
                            TokenReference::symbol("___KALEDIS_REPORT_VARIABLE").unwrap();
                        let prefix = Prefix::Name(name_token);

                        let mut args: Punctuated<Expression> = Punctuated::new();

                        // The line that is happening
                        args.push(Pair::Punctuated(
                            Expression::Number(
                                TokenReference::symbol(&pos.line().to_string()).unwrap(),
                            ),
                            TokenReference::symbol(",").unwrap(),
                        ));

                        // The column that is happening
                        args.push(Pair::Punctuated(
                            Expression::Number(
                                TokenReference::symbol(&pos.line().to_string()).unwrap(),
                            ),
                            TokenReference::symbol(",").unwrap(),
                        ));

                        // The file that is happening
                        args.push(Pair::Punctuated(
                            Expression::String(
                                TokenReference::symbol(&self.file_path.to_string_lossy()).unwrap(),
                            ),
                            TokenReference::symbol(",").unwrap(),
                        ));

                        // The new value
                        args.push(Pair::End(Expression::Var(var.clone())));
                        let func_args = FunctionArgs::Parentheses {
                            parentheses: ContainedSpan::new(
                                TokenReference::symbol("(").unwrap(),
                                TokenReference::symbol(")").unwrap(),
                            ),
                            arguments: args,
                        };
                        let func_call = FunctionCall::new(prefix)
                            .with_suffixes(vec![Suffix::Call(Call::AnonymousCall(func_args))]);

                        new_statements.push((Stmt::FunctionCall(func_call), None));
                    }
                }
            }
        }

        node.with_stmts(new_statements)
    }
}

impl ReportVariables {}
