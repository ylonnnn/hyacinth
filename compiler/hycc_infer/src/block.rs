use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{block::HirBlock, stmt::HirStmtKind};
use hycc_ty::{context::TyId, ty::Ty};

use crate::inferer::{InferResult, TyInferer};

impl<'t, 'd, 'r> TyInferer<'t, 'd, 'r> {
    pub(crate) fn infer_block(&mut self, block: &HirBlock) -> InferResult<TyId> {
        let mut ty: Option<Ty> = self.tctx.get_ty_of_hir(block.id).cloned();

        for stmt in &block.stmts {
            if let Err(Some(diag)) = self.infer_stmt(&stmt) {
                self.dctx.add(diag);
            }

            let HirStmtKind::Pass(_) = &stmt.kind else {
                continue;
            };

            if let Some(ty) = &ty {
                let Some(stmt_ty) = self.tctx.get_ty_of_hir(stmt.id).cloned() else {
                    continue;
                };

                self.check(&ty, &stmt_ty).map(|diag| self.dctx.add(diag));
            } else {
                if let Some(stmt_ty) = self.tctx.get_ty_of_hir(stmt.id).cloned() {
                    ty.replace(stmt_ty);
                }
            }
        }

        Ok(ty.map(|ty| ty.id).unwrap_or(self.tctx.make_unit_ty()))
    }
}
