use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{block::HirBlock, stmt::HirStmtKind};
use hycc_ty::{context::TyId, ty::Ty};

use crate::inferer::{InferResult, TyInferer};

impl<'t, 'd, 'r> TyInferer<'t, 'd, 'r> {
    pub(crate) fn infer_block(&mut self, block: &HirBlock) -> InferResult<TyId> {
        let expected_ty: Option<Ty> = self.tctx.get_ty_of_hir(block.id).cloned();
        let mut ty: Option<Ty> = None;

        for stmt in &block.stmts {
            if let Err(Some(diag)) = self.infer_stmt(&stmt) {
                self.dctx.add(diag);
            }

            let HirStmtKind::Pass(_) = &stmt.kind else {
                continue;
            };

            let Some(stmt_ty) = self.tctx.get_ty_of_hir(stmt.id).cloned() else {
                continue;
            };

            if ty.is_none() {
                ty.replace(stmt_ty);
            }

            if let Some(ty) = &ty {
                let Some(expected_ty) = &expected_ty else {
                    continue;
                };

                self.check(&expected_ty, &ty)
                    .map(|diag| self.dctx.add(diag));
            }
        }

        Ok(ty.map(|ty| ty.id).unwrap_or(self.tctx.make_unit_ty()))
    }
}
