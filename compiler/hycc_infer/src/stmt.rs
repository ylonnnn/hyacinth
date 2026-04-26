use hycc_hir::stmt::{HirStmt, HirStmtKind};

use crate::inferer::{InferResult, TyInferer};

impl<'t, 'i, 'd, 'r, 'h> TyInferer<'t, 'i, 'd, 'r, 'h> {
    pub(crate) fn infer_stmt(&mut self, stmt: &HirStmt) -> InferResult {
        match &stmt.kind {
            HirStmtKind::Item(item) => self.infer_item(&item),
            HirStmtKind::Expr(expr) => {
                self.infer_expr(&expr)?;
                Ok(())
            }
        }
    }
}
