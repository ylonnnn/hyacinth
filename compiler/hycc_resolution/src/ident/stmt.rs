use hycc_hir::stmt::{HirStmt, HirStmtKind};

use crate::ident::resolver::{ResolveResult, Resolver};

impl<'s, 'd> Resolver<'s, 'd> {
    pub(crate) fn resolve_stmt(&mut self, stmt: &HirStmt) -> ResolveResult {
        match &stmt.kind {
            HirStmtKind::Item(item) => self.resolve_item(&item),
            HirStmtKind::Expr(expr) => self.resolve_expr(&expr),
        }
    }
}
