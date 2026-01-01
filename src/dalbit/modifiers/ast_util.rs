use full_moon::{
    ast::{span::ContainedSpan, BinOp, Expression, UnOp},
    tokenizer::{Token, TokenReference, TokenType},
    ShortString,
};

#[inline]
pub fn create_parentheses(inner_exp: Expression, contained: Option<ContainedSpan>) -> Expression {
    let contained = if let Some(contained) = contained {
        contained
    } else {
        ContainedSpan::new(
            TokenReference::symbol("(").unwrap(),
            TokenReference::symbol(")").unwrap(),
        )
    };
    Expression::Parentheses {
        contained,
        expression: Box::new(inner_exp),
    }
}

#[inline]
pub fn create_binary_operator(left: Expression, binop: BinOp, right: Expression) -> Expression {
    Expression::BinaryOperator {
        lhs: Box::new(left),
        binop: binop,
        rhs: Box::new(right),
    }
}

#[inline]
pub fn create_unary_operator(unop: UnOp, exp: Expression) -> Expression {
    Expression::UnaryOperator {
        unop,
        expression: Box::new(exp),
    }
}

#[inline]
pub fn create_number<T: Into<String> + AsRef<str>>(number_text: T) -> Expression {
    Expression::Number(TokenReference::new(
        Vec::new(),
        Token::new(TokenType::Number {
            text: ShortString::new(number_text),
        }),
        Vec::new(),
    ))
}
