use hycc_hir::stmt::{HirStmt, HirStmtKind};

use crate::inferer::{InferResult, TyInferer};

impl<'t, 'd, 'r>  TyInferer<'t, 'd, 'r> {
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
