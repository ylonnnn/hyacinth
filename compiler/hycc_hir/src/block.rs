use hycc_span::Span;

use crate::{HirId, stmt::HirStmt};

#[derive(Debug, Clone)]
pub struct HirBlock {
    pub id: HirId,
    pub stmts: Vec<HirStmt>,
    pub span: Span,
}
