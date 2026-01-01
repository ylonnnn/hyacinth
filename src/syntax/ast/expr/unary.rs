use crate::syntax::{Expr, Token};

#[derive(Debug)]
pub struct UnaryExpr {
    pub op: Token,
    pub expr: Box<Expr>,
}
