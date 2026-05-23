use crate::{Block, Expr, Item, token::Token};

use hycc_span::Span;

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum StmtKind {
    If(Box<IfStmt>),

    Ret(Box<RetStmt>),
    Pass(Box<PassStmt>),

    Item(Box<Item>),
    Expr(Box<Expr>),
}

impl StmtKind {
    pub fn span(&self) -> Span {
        match self {
            Self::If(ite) => ite.span(),
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
pub struct IfStmt {
    pub cond: Box<Expr>,
    pub consequent: Box<Block>,
    pub alternate: Option<Box<Block>>,
}

impl IfStmt {
    pub fn span(&self) -> Span {
        self.cond.span.merge(
            self.alternate
                .as_ref()
                .map(|alt| &alt.span)
                .unwrap_or(&self.consequent.span),
        )
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
