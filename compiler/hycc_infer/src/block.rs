use hycc_diagnostic::diagnostic::{Diagnostics, FromResultEmitter};
use hycc_hir::{block::HirBlock, stmt::HirStmtKind};
use hycc_ty::{ctx::TyId, ty::Ty};

use crate::{diag::InferResult, inferer::TyInferer};

impl<'i, 'h> TyInferer<'i, 'h> {
    pub(crate) fn check_block(&mut self, block: &HirBlock) -> InferResult {
        block
            .stmts
            .iter()
            .for_each(|stmt| self.check_stmt(&stmt).emit_discard(&mut self.dctx));

        Ok(())
    }

    pub(crate) fn infer_block(&mut self, block: &HirBlock) -> InferResult<TyId> {
        let expected_ty = self.tctx.get_hir_ty(block.id).cloned();
        self.tctx.dettach_hir(block.id);

        for stmt in &block.stmts {
            self.infer_stmt(&stmt).emit(&mut self.dctx);

            match &stmt.kind {
                HirStmtKind::Ret(_) => {
                    if self.tctx.get_hir_ty(block.id).is_none() {
                        let never_ty = self.tctx.make_never_ty();
                        self.tctx
                            .attach_to_hir(block.id, Ty::new(never_ty, block.span));
                    }
                }

                HirStmtKind::Pass(_) => {
                    let Some(stmt_ty) = self.tctx.get_hir_ty(stmt.id).cloned() else {
                        continue;
                    };

                    if self.tctx.get_hir_ty(block.id).is_none() {
                        self.tctx.attach_to_hir(block.id, stmt_ty.clone());
                    }

                    expected_ty
                        .as_ref()
                        .map(|expected_ty| self.check(&expected_ty, &stmt_ty).emit(&mut self.dctx));
                }

                _ => {}
            }
        }

        let unit_ty = self.tctx.make_unit_ty();
        let ty_id = self
            .tctx
            .get_hir_ty(block.id)
            .map_or(unit_ty, |ty| expected_ty.map_or(ty.id, |ty| ty.id));

        Ok(ty_id)
    }
}
