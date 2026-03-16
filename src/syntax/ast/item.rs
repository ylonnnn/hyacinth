use crate::syntax::{Expr, SpannedNode, Token, Type};

#[derive(Debug, Clone)]
pub enum Item {
    Variable(VariableDeclStmt),
}

#[derive(Debug, Clone)]
pub struct VariableDeclStmt {
    pub ident: Token,
    pub ty: Option<SpannedNode<Type>>,
    pub value: Option<SpannedNode<Expr>>,
}
