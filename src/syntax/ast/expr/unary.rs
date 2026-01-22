use crate::syntax::{Expr, Token};

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: Token,
    pub expr: Box<Expr>,
}
