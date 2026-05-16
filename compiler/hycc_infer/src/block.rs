use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{block::HirBlock, stmt::HirStmtKind};
use hycc_ty::{context::TyId, ty::Ty};

use crate::inferer::{InferResult, TyInferer};

impl<'t, 'd, 'c, 'h> TyInferer<'t, 'd, 'c, 'h> {
    pub(crate) fn infer_block(&mut self, block: &HirBlock) -> InferResult<TyId> {
        let expected_ty: Option<Ty> = self.tctx.get_ty_of_hir(block.id).cloned();
        self.tctx.dettach_hir(block.id);

        for stmt in &block.stmts {
            if let Err(Some(diag)) = self.infer_stmt(&stmt) {
                self.dctx.add(diag);
            }

            match &stmt.kind {
                HirStmtKind::Ret(_) => {
                    if self.tctx.get_ty_of_hir(block.id).is_none() {
                        let never_ty = self.tctx.make_never_ty();
                        self.tctx
                            .attach_to_hir(block.id, Ty::new(never_ty, block.span));
                    }
                }

                HirStmtKind::Pass(_) => {
                    let Some(stmt_ty) = self.tctx.get_ty_of_hir(stmt.id).cloned() else {
                        continue;
                    };

                    dbg!(&stmt_ty);

                    if self.tctx.get_ty_of_hir(block.id).is_none() {
                        self.tctx.attach_to_hir(block.id, stmt_ty.clone());
                    }

                    let Some(expected_ty) = &expected_ty else {
                        continue;
                    };

                    self.check(&expected_ty, &stmt_ty)
                        .map(|diag| self.dctx.add(diag));
                }

                _ => {}
            }
        }

        let unit_ty = self.tctx.make_unit_ty();
        let ty_id = self
            .tctx
            .get_ty_of_hir(block.id)
            .map(|ty| expected_ty.map(|ty| ty.id).unwrap_or(ty.id))
            .unwrap_or(unit_ty);

        Ok(ty_id)
    }
}
