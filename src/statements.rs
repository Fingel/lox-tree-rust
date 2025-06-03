use crate::expressions::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression(Box<Expr>),
    Print(Box<Expr>),
}
