use crate::syntax::{Path, SpannedNode, Token};

#[derive(Debug, Clone)]
pub enum Expr {
    Path(Path),
    Literal(LiteralExpr),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
}

#[derive(Debug, Clone)]
pub enum LiteralExpr {
    Int(i64),
    Float(f64),
    Bool(bool),
    // TODO: Add other literal expression types
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: Token,
    pub expr: Box<SpannedNode<Expr>>,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub op: Token,
    pub left: Box<SpannedNode<Expr>>,
    pub rigth: Box<SpannedNode<Expr>>,
}
