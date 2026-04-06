use hycc_span::Span;

use crate::{HirId, stmt::HirStmt};

#[derive(Debug, Clone)]
pub struct HirBlock<'h> {
    pub id: HirId,
    pub stmts: Vec<&'h HirStmt<'h>>,
    pub span: Span,
}

impl<'h> HirBlock<'h> {
    pub fn new(stmts: Vec<&'h HirStmt<'h>>, span: Span) -> Self {
        Self {
            id: HirId::Invalid,
            stmts,
            span,
        }
    }
}
