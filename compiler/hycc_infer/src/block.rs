use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{block::HirBlock, stmt::HirStmtKind};
use hycc_ty::context::TyId;

use crate::inferer::{InferResult, TyInferer};

impl<'t, 'd, 'r> TyInferer<'t, 'd, 'r> {
    pub(crate) fn infer_block(&mut self, block: &HirBlock) -> InferResult<TyId> {
        let mut ty_id = self.tctx.make_unit_ty();
        for stmt in &block.stmts {
            if let Err(Some(diag)) = self.infer_stmt(&stmt) {
                self.dctx.add(diag);
            }

            let HirStmtKind::Pass(_) = &stmt.kind else {
                continue;
            };

            if let Some(ty) = self.tctx.get_ty_of_hir(stmt.id) {
                ty_id = ty.id;
            }
        }

        Ok(ty_id)
    }
}
