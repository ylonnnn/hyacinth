use hycc_span::Span;

use crate::{HirId, expr::HirExpr, item::HirItem};

#[derive(Debug, Clone)]
pub enum HirStmtKind<'h> {
    Expr(&'h HirExpr<'h>),
    Item(&'h HirItem<'h>),
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
