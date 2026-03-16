use crate::{
    core::Span,
    syntax::{Expr, Item},
};

#[derive(Debug, Clone)]
pub enum StmtKind {
    Expr(Expr),
    Item(Item),
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}
