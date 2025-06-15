use crate::tokens::{Object, Token};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Unary(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Token, Vec<Expr>),
    Grouping(Box<Expr>),
    Literal(Object),
    Variable(Token),
    Assignment(Token, Box<Expr>),
    Logical(Box<Expr>, Token, Box<Expr>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Binary(left, operator, right) => {
                write!(f, "{}", parenthesize(&operator.lexeme, &[left, right]))
            }
            Expr::Grouping(expression) => write!(f, "{}", parenthesize("group", &[expression])),
            Expr::Literal(value) => write!(f, "{}", value),
            Expr::Unary(operator, right) => {
                write!(f, "{}", parenthesize(&operator.lexeme, &[right]))
            }
            Expr::Variable(token) => write!(f, "{}", token.lexeme),
            Expr::Assignment(token, expr) => write!(f, "{} = {}", &token.lexeme, expr),
            Expr::Logical(left, operator, right) => {
                write!(f, "{}", parenthesize(&operator.lexeme, &[left, right]))
            }
            Expr::Call(callee, paren, args) => {
                let refs: Vec<&Expr> = args.iter().collect();
                write!(f, "{}{}{}", callee, paren, parenthesize("call", &refs))
            }
        }
    }
}

fn parenthesize(name: &str, expressions: &[&Expr]) -> String {
    let mut result = String::new();
    result.push_str(&format!("({}", name));
    for expr in expressions.iter() {
        result.push(' ');
        result.push_str(&format!("{}", expr));
    }
    result.push(')');
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::TokenType;

    #[test]
    fn test_simple_expr() {
        let expr = Expr::Binary(
            Box::new(Expr::Literal(Object::Number(1.0))),
            Token::new(TokenType::Plus, "+".to_string(), None, 1),
            Box::new(Expr::Literal(Object::Number(2.0))),
        );
        assert_eq!(format!("{}", expr), "(+ 1 2)");
    }

    #[test]
    fn test_book_expr() {
        let expr = Expr::Binary(
            Box::new(Expr::Unary(
                Token::new(TokenType::Minus, "-".to_string(), None, 1),
                Box::new(Expr::Literal(Object::Number(123.0))),
            )),
            Token::new(TokenType::Star, "*".to_string(), None, 1),
            Box::new(Expr::Grouping(Box::new(Expr::Literal(Object::Number(
                45.67,
            ))))),
        );
        assert_eq!(format!("{}", expr), "(* (- 123) (group 45.67))");
    }
}
