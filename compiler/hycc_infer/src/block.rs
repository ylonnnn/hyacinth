use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{block::HirBlock, stmt::HirStmtKind};
use hycc_ty::{context::TyId, ty::Ty};

use crate::inferer::{InferResult, TyInferer};

impl<'t, 'd, 'r> TyInferer<'t, 'd, 'r> {
    pub(crate) fn infer_block(&mut self, block: &HirBlock) -> InferResult<TyId> {
        let expected_ty: Option<Ty> = self.tctx.get_ty_of_hir(block.id).cloned();
        self.tctx.dettach_hir(block.id);

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

            if self.tctx.get_ty_of_hir(block.id).is_none() {
                self.tctx.attach_to_hir(block.id, stmt_ty.clone());
            }

            let Some(expected_ty) = &expected_ty else {
                continue;
            };

            self.check(&expected_ty, &stmt_ty)
                .map(|diag| self.dctx.add(diag));
        }

        let unit_ty = self.tctx.make_unit_ty();
        let ty_id = self
            .tctx
            .get_ty_of_hir(block.id)
            .map(|_| expected_ty.map(|ty| ty.id).unwrap_or(unit_ty))
            .unwrap_or(unit_ty);

        Ok(ty_id)
    }
}
