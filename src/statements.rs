use crate::{expressions::Expr, tokens::Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Print(Box<Expr>),
    Block(Vec<Stmt>),
    Expression(Box<Expr>),
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    Var(Token, Option<Box<Expr>>),
    While(Box<Expr>, Box<Stmt>),
}
