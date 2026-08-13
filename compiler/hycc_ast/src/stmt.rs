use crate::{block::Block, expr::Expr, item::Item, token::Token};

use hycc_span::Span;

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum StmtKind {
    Ret(Box<RetStmt>),
    Pass(Box<PassStmt>),

    Item(Box<Item>),
    Expr(Box<Expr>),
}

impl StmtKind {
    pub fn span(&self) -> Span {
        match self {
            Self::Ret(ret) => ret.span,
            Self::Pass(pass) => pass.span,
            Self::Item(item) => item.span,
            Self::Expr(expr) => expr.span,
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

#[derive(Debug, Clone)]
pub struct RetStmt {
    pub value: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct PassStmt {
    pub value: Option<Box<Expr>>,
    pub label: Option<Token>,
    pub span: Span,
}
