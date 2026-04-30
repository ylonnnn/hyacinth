use hycc_diagnostic::DiagnosticContext;
use hycc_hir::stmt::{HirStmt, HirStmtKind};
use hycc_span::Span;
use hycc_ty::ty::{Ty, TyKind};
use hycc_util::bug;

use crate::inferer::{InferResult, TyInferer};

impl<'t, 'd, 'r> TyInferer<'t, 'd, 'r> {
    pub(crate) fn infer_stmt(&mut self, stmt: &HirStmt) -> InferResult {
        match &stmt.kind {
            HirStmtKind::Ret(ret) => {
                let Some(fn_ctx) = &self.fn_ctx else {
                    bug!("fn ctx must exist when entering a function")
                };

                let TyKind::Fn(fn_ty) = self.tctx.get(fn_ctx.ty.id) else {
                    bug!("fn ty must be the ty of the function")
                };

                let ret_ty = fn_ty.ret_ty;

                if let Some(val) = ret.value {
                    let val_ty = match self.infer_expr(&val) {
                        Ok(ty_id) => ty_id,
                        Err(diag) => {
                            diag.map(|diag| self.dctx.add(diag));
                            return Ok(());
                        }
                    };

                    self.check(
                        &Ty::new(ret_ty, Span::default()),
                        &Ty::new(val_ty, val.span),
                    )
                    .map(|diag| self.dctx.add(diag));
                }

                Ok(())
            }

            HirStmtKind::Pass(pass) => {
                if let Some(value) = pass.value {
                    match self.infer_expr(&value) {
                        Ok(ty_id) => self.tctx.attach_to_hir(stmt.id, Ty::new(ty_id, value.span)),
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
