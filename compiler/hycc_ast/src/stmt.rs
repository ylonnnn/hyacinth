use crate::{Expr, Item};

use hycc_span::Span;

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum StmtKind {
    Expr(Box<Expr>),
    Item(Box<Item>),
}

impl StmtKind {
    pub fn span(&self) -> Span {
        match self {
            Self::Expr(expr) => expr.span,
            Self::Item(item) => item.span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

impl Stmt {
    pub fn new(kind: StmtKind) -> Self {
        Self {
            span: kind.span(),
            kind,
        }
    }
}
