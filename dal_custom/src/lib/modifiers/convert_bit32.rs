use std::string::ToString;
use std::{collections::HashMap, str::FromStr};

use full_moon::ast::punctuated::Pair;
use full_moon::ast::Do;
use full_moon::node::Node;
use full_moon::tokenizer::{Symbol, Token, TokenType};
use full_moon::ShortString;
use full_moon::{
    ast::{
        punctuated::Punctuated, BinOp, Call, Expression, FunctionArgs, FunctionCall, Index, Stmt,
        Suffix, UnOp, Var,
    },
    tokenizer::TokenReference,
    visitors::VisitorMut,
};
use strum_macros::{Display, EnumString};

use super::ast_util;

pub const CONVERT_BIT32_MODIFIER_NAME: &str = "convert_bit32";
const DEFAULT_BIT32_IDENTIFIER: &str = "bit32";
const MASKING_NUMBER_TOKEN_SYMBOL: &str = "0xFFFFFFFF";

#[inline]
fn mask_32bit(exp: Expression) -> Expression {
    ast_util::create_binary_operator(
        exp,
        BinOp::Ampersand(TokenReference::symbol("&").unwrap()),
        ast_util::create_number(MASKING_NUMBER_TOKEN_SYMBOL),
    )
}

fn index_to_string(index: &Index) -> Option<String> {
    match index {
        Index::Brackets {
            brackets: _,
            expression,
        } => {
            let mut string = expression.to_string();
            string.remove(0);
            string.pop();
            Some(string)
        }
        Index::Dot { dot: _, name: _ } => {
            let mut string = index.to_string();
            string.remove(0);
            Some(string)
        }
        _ => None,
    }
}

#[derive(Debug, Display, EnumString)]
enum Bit32Method {
    #[strum(serialize = "rshift")]
    RightShift,
    #[strum(serialize = "lshift")]
    LeftShift,
    #[strum(serialize = "band")]
    And,
    #[strum(serialize = "bor")]
    Or,
    #[strum(serialize = "bxor")]
    Xor,
    #[strum(serialize = "bnot")]
    Not,
    #[strum(serialize = "btest")]
    Test,
}

impl Bit32Method {
    fn convert(&self, call: &Call) -> Option<Expression> {
        if let Call::AnonymousCall(args) = call {
            if let FunctionArgs::Parentheses {
                parentheses,
                arguments,
            } = args
            {
                let mut iter = arguments.iter();
                let first_arg = iter.next()?;

                let binop = match self {
                    Bit32Method::RightShift => {
                        BinOp::DoubleGreaterThan(TokenReference::symbol(">>").unwrap())
                    }
                    Bit32Method::LeftShift => {
                        BinOp::DoubleLessThan(TokenReference::symbol("<<").unwrap())
                    }
                    Bit32Method::And => BinOp::Ampersand(TokenReference::symbol("&").unwrap()),
                    Bit32Method::Or => BinOp::Pipe(TokenReference::symbol("|").unwrap()),
                    Bit32Method::Xor => BinOp::Tilde(TokenReference::symbol("~").unwrap()),
                    Bit32Method::Not => {
                        let masking_arg = mask_32bit(first_arg.clone());
                        let parenthese = ast_util::create_parentheses(masking_arg, None);
                        let bnot_exp = ast_util::create_unary_operator(
                            UnOp::Tilde(TokenReference::symbol("~").unwrap()),
                            parenthese,
                        );
                        let masking_unop = mask_32bit(bnot_exp);
                        return Some(ast_util::create_parentheses(
                            masking_unop,
                            Some(parentheses.clone()),
                        ));
                    }
                    Bit32Method::Test => {
                        let second_arg = iter.next()?;
                        let band_exp = ast_util::create_binary_operator(
                            first_arg.clone(),
                            BinOp::Ampersand(TokenReference::symbol("&").unwrap()),
                            second_arg.clone(),
                        );
                        let parenthese = ast_util::create_parentheses(band_exp, None);
                        let masking_bin_exp = mask_32bit(parenthese);
                        let not_equal_exp = ast_util::create_binary_operator(
                            masking_bin_exp,
                            BinOp::TildeEqual(TokenReference::symbol("~=").unwrap()),
                            ast_util::create_number("0"),
                        );
                        return Some(ast_util::create_parentheses(
                            not_equal_exp,
                            Some(parentheses.clone()),
                        ));
                    }
                };

                let second_arg = iter.next()?;
                let bitop_exp =
                    ast_util::create_binary_operator(first_arg.clone(), binop, second_arg.clone());
                let parenthese = ast_util::create_parentheses(bitop_exp, None);
                let masking_bin_exp = mask_32bit(parenthese);
                return Some(ast_util::create_parentheses(
                    masking_bin_exp,
                    Some(parentheses.clone()),
                ));
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct ConvertBit32 {
    bit32_identifier: String,
    bit32_methods: HashMap<String, Bit32Method>,
}

impl Default for ConvertBit32 {
    fn default() -> Self {
        Self {
            bit32_identifier: DEFAULT_BIT32_IDENTIFIER.to_owned(),
            bit32_methods: HashMap::new(),
        }
    }
}

impl VisitorMut for ConvertBit32 {
    fn visit_stmt(&mut self, stmt: Stmt) -> Stmt {
        match &stmt {
            Stmt::FunctionCall(func_call) => {
                if let Some(_) = self.convert(func_call) {
                    return Stmt::Do(
                        Do::new()
                            .with_do_token(TokenReference::new(
                                func_call
                                    .surrounding_trivia()
                                    .0
                                    .into_iter()
                                    .cloned()
                                    .collect(),
                                Token::new(TokenType::Symbol { symbol: Symbol::Do }),
                                vec![Token::new(TokenType::Whitespace {
                                    characters: ShortString::new(" "),
                                })],
                            ))
                            .with_end_token(TokenReference::new(
                                Vec::new(),
                                Token::new(TokenType::Symbol {
                                    symbol: Symbol::End,
                                }),
                                func_call
                                    .surrounding_trivia()
                                    .1
                                    .into_iter()
                                    .cloned()
                                    .collect(),
                            )),
                    );
                }
            }
            Stmt::Assignment(assign) => {
                if self.check_replaced(assign.variables(), assign.expressions()) {
                    let mut do_trailing_trivia: Vec<Token> = Vec::new();
                    for token_ref in assign.tokens() {
                        for t in token_ref.leading_trivia() {
                            do_trailing_trivia.push(t.to_owned());
                        }
                        for t in token_ref.trailing_trivia() {
                            do_trailing_trivia.push(t.to_owned());
                        }
                    }
                    return Stmt::Do(
                        Do::new()
                            .with_do_token(TokenReference::new(
                                vec![Token::new(TokenType::Whitespace {
                                    characters: ShortString::new(" "),
                                })],
                                Token::new(TokenType::Symbol { symbol: Symbol::Do }),
                                do_trailing_trivia,
                            ))
                            .with_end_token(TokenReference::new(
                                vec![Token::new(TokenType::Whitespace {
                                    characters: ShortString::new(" "),
                                })],
                                Token::new(TokenType::Symbol {
                                    symbol: Symbol::End,
                                }),
                                vec![Token::new(TokenType::Whitespace {
                                    characters: ShortString::new(" "),
                                })],
                            )),
                    );
                }
            }
            Stmt::LocalAssignment(local_assign) => {
                let mut variables: Punctuated<Var> = Punctuated::new();
                for token in local_assign.names() {
                    variables.push(Pair::new(Var::Name(token.clone()), None));
                }
                if self.check_replaced(&variables, local_assign.expressions()) {
                    let mut do_trailing_trivia: Vec<Token> = Vec::new();
                    for token_ref in local_assign.tokens() {
                        for t in token_ref.leading_trivia() {
                            do_trailing_trivia.push(t.to_owned());
                        }
                        for t in token_ref.trailing_trivia() {
                            do_trailing_trivia.push(t.to_owned());
                        }
                    }
                    return Stmt::Do(
                        Do::new()
                            .with_do_token(TokenReference::new(
                                vec![Token::new(TokenType::Whitespace {
                                    characters: ShortString::new(" "),
                                })],
                                Token::new(TokenType::Symbol { symbol: Symbol::Do }),
                                do_trailing_trivia,
                            ))
                            .with_end_token(TokenReference::new(
                                Vec::new(),
                                Token::new(TokenType::Symbol {
                                    symbol: Symbol::End,
                                }),
                                vec![Token::new(TokenType::Whitespace {
                                    characters: ShortString::new(" "),
                                })],
                            )),
                    );
                }
            }
            _ => {}
        }
        stmt
    }

    /// To convert bit32 methods/calls and linked identifiers into bitwise operators
    ///
    /// Conversion Example: `local x = bit32.band; local y = x(1, 2)` -> `do then; local y = ((1&2)&0xFFFFFFFF)`
    fn visit_expression(&mut self, exp: Expression) -> Expression {
        if let Expression::FunctionCall(func_call) = &exp {
            if let Some(exp) = self.convert(func_call) {
                return exp;
            }
        }
        exp
    }
}

impl ConvertBit32 {
    #[inline]
    fn is_bit32_identifier(&self, string: impl Into<String>) -> bool {
        string.into() == self.bit32_identifier
    }

    fn check_replaced(
        &mut self,
        variables: &Punctuated<Var>,
        expressions: &Punctuated<Expression>,
    ) -> bool {
        for (var, exp) in variables.iter().zip(expressions.iter()) {
            // local x = bit32.band
            if let Expression::Var(exp) = exp {
                match exp {
                    Var::Expression(var_exp) => {
                        if !self.is_bit32_identifier(var_exp.prefix().to_string()) {
                            return false;
                        }
                        let mut iter = var_exp.suffixes();
                        let first = iter.next();
                        if let Some(first) = first {
                            // there's an index(ex. `band`)
                            if let Suffix::Index(index) = first {
                                let index = index_to_string(index);
                                if let Some(index) = index {
                                    if let Ok(method) = Bit32Method::from_str(index.trim()) {
                                        self.bit32_methods
                                            .insert(var.to_string().trim().to_owned(), method);
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                    Var::Name(_) => {
                        if self.is_bit32_identifier(exp.to_string().trim().to_owned()) {
                            self.bit32_identifier = var.to_string().trim().to_owned();
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }
        false
    }

    fn convert(&mut self, func_call: &FunctionCall) -> Option<Expression> {
        let mut iter = func_call.suffixes();
        let first = iter.next();
        let second = iter.next();
        let prefix = func_call.prefix().to_string();
        match (first, second) {
            (Some(first), Some(second)) => {
                if !self.is_bit32_identifier(prefix) {
                    return None;
                }
                match (first, second) {
                    // there's another index(ex. `band`) before a call(ex. `(1, 2)`)
                    (Suffix::Index(index), Suffix::Call(call)) => {
                        let index = index_to_string(index)?;
                        if let Ok(method) = Bit32Method::from_str(index.trim()) {
                            return method.convert(call);
                        }
                        None
                    }
                    _ => None,
                }
            }
            (Some(first), None) => {
                // there's only a call(ex. `(1, 2)`)
                if let Suffix::Call(call) = first {
                    if let Some(method) = self.bit32_methods.get(&prefix) {
                        return method.convert(call);
                    }
                }
                None
            }
            _ => None,
        }
    }
}
