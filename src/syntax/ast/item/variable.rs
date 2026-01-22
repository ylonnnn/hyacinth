use crate::syntax::{Expr, GenNode, Token};

#[derive(Debug, Clone)]
pub struct VariableDeclStmt {
    pub ident: Token,
    pub value: Option<GenNode<Expr>>,
}
