use crate::tokens::{Literal, Token, TokenType};
use std::fmt;

pub enum Expr {
    // TODO see what this looks like with tuples instead of structs
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Literal,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => write!(f, "{}", parenthesize(&operator.lexeme, &[left, right])),
            Expr::Grouping { expression } => write!(f, "{}", parenthesize("group", &[expression])),
            Expr::Literal { value } => write!(f, "{}", value),
            Expr::Unary { operator, right } => {
                write!(f, "{}", parenthesize(&operator.lexeme, &[right]))
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

    #[test]
    fn test_simple_expr() {
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal {
                value: Literal::Number(1.0),
            }),
            operator: Token::new(TokenType::Plus, "+".to_string(), None, 1),
            right: Box::new(Expr::Literal {
                value: Literal::Number(2.0),
            }),
        };
        assert_eq!(format!("{}", expr), "(+ 1 2)");
    }

    #[test]
    fn test_book_expr() {
        let expr = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: Token::new(TokenType::Minus, "-".to_string(), None, 1),
                right: Box::new(Expr::Literal {
                    value: Literal::Number(123.0),
                }),
            }),
            operator: Token::new(TokenType::Star, "*".to_string(), None, 1),
            right: Box::new(Expr::Grouping {
                expression: Box::new(Expr::Literal {
                    value: Literal::Number(45.67),
                }),
            }),
        };
        assert_eq!(format!("{}", expr), "(* (- 123) (group 45.67))");
    }
}
