use crate::syntax::{Expr, Token};

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub op: Token,
    pub left: Box<Expr>,
    pub rigth: Box<Expr>,
}
