use crate::syntax::{Expr, Token};

#[derive(Debug)]
pub struct BinaryExpr {
    pub op: Token,
    pub left: Box<Expr>,
    pub rigth: Box<Expr>,
}
