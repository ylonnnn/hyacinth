use crate::{
    core::Span,
    syntax::{Path, Token},
};

#[derive(Debug, Clone)]
pub enum ExprKind {
    Path(Path),
    Literal(LiteralExpr),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum LiteralExprKind {
    Int,
    Float,
    Bool,
    // TODO: Add other literal expression types
}

#[derive(Debug, Clone)]
pub struct LiteralExpr {
    pub kind: LiteralExprKind,
    pub token: Token,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: Token,
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub op: Token,
    pub left: Box<Expr>,
    pub rigth: Box<Expr>,
}
