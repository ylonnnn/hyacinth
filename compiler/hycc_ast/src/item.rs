use crate::{
    core::Span,
    syntax::{Expr, Token, Ty},
};

#[derive(Debug, Clone)]
pub enum ItemKind {
    VarDecl(VarDeclStmt),
}

#[derive(Debug, Clone)]
pub struct Item {
    pub kind: ItemKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct VarDeclStmt {
    pub ident: Token,
    pub ty: Option<Ty>,
    pub value: Option<Expr>,
}
