use hycc_hir::stmt::{HirStmt, HirStmtKind};

use crate::{ResolveResult, ty::resolver::TyResolver};

impl<'d, 'r> TyResolver<'d, 'r> {
    pub(crate) fn resolve_stmt(&mut self, stmt: &HirStmt) -> ResolveResult {
        match &stmt.kind {
            HirStmtKind::Item(item) => self.resolve_item(&item),
            _ => Ok(()),
        }
    }
}
