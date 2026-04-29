use hycc_diagnostic::DiagnosticContext;
use hycc_hir::stmt::{HirStmt, HirStmtKind};
use hycc_ty::ty::Ty;

use crate::inferer::{InferResult, TyInferer};

impl<'t, 'd, 'r> TyInferer<'t, 'd, 'r> {
    pub(crate) fn infer_stmt(&mut self, stmt: &HirStmt) -> InferResult {
        match &stmt.kind {
            HirStmtKind::Ret(ret) => todo!("infer ret stmt"),
            HirStmtKind::Pass(pass) => {
                if let Some(value) = pass.value {
                    match self.infer_expr(&value) {
                        Ok(ty_id) => self.tctx.attach_to_hir(stmt.id, Ty::new(ty_id, stmt.span)),
                        Err(diag) => {
                            diag.map(|diag| self.dctx.add(diag));
                        }
                    }
                }

                Ok(())
            }

            HirStmtKind::Item(item) => self.infer_item(&item),
            HirStmtKind::Expr(expr) => {
                self.infer_expr(&expr)?;
                Ok(())
            }
        }
    }
}
