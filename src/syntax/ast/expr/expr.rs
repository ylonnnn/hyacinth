use crate::syntax::{BinaryExpr, LiteralExpr, UnaryExpr};

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(LiteralExpr),
    Identifier(String),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
}
