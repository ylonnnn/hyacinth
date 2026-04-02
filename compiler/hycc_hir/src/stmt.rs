use hycc_span::Span;

use crate::{HirId, expr::HirExpr, item::HirItem};

#[derive(Debug, Clone)]
pub enum HirStmtKind {
    Expr(Box<HirExpr>),
    Item(Box<HirItem>),
}

#[derive(Debug, Clone)]
pub struct HirStmt {
    pub id: HirId,
    pub kind: HirStmtKind,
    pub span: Span,
}
