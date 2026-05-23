use hycc_span::Span;

use crate::{HirId, block::HirBlock, expr::HirExpr, item::HirItem, path::HirRawIdent};

#[derive(Debug, Clone)]
pub enum HirStmtKind<'h> {
    If(Box<HirIfStmt<'h>>),

    Ret(Box<HirRetStmt<'h>>),
    Pass(Box<HirPassStmt<'h>>),

    Item(&'h HirItem<'h>),
    Expr(&'h HirExpr<'h>),
}

#[derive(Debug, Clone)]
pub struct HirStmt<'h> {
    pub id: HirId,
    pub kind: HirStmtKind<'h>,
    pub span: Span,
}

impl<'h> HirStmt<'h> {
    pub fn new(kind: HirStmtKind<'h>, span: Span) -> Self {
        Self {
            id: HirId::Invalid,
            kind,
            span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HirIfStmt<'h> {
    pub cond: &'h HirExpr<'h>,
    pub consequent: &'h HirBlock<'h>,
    pub alternate: Option<&'h HirBlock<'h>>,
}

#[derive(Debug, Clone)]
pub struct HirRetStmt<'h> {
    pub value: Option<&'h HirExpr<'h>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirPassStmt<'h> {
    pub value: Option<&'h HirExpr<'h>>,
    pub label: Option<&'h HirRawIdent>,
    pub span: Span,
}
