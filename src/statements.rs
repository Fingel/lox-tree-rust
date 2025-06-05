use crate::{expressions::Expr, tokens::Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Print(Box<Expr>),
    Expression(Box<Expr>),
    Var(Token, Box<Option<Expr>>),
}
