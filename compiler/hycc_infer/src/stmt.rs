use hycc_diagnostic::DiagnosticContext;
use hycc_hir::stmt::{HirStmt, HirStmtKind};
use hycc_span::Span;
use hycc_ty::ty::{Ty, TyKind};
use hycc_util::bug;

use crate::inferer::{InferResult, TyInferer};

impl<'t, 'd, 'c> TyInferer<'t, 'd, 'c> {
    pub(crate) fn infer_stmt(&mut self, stmt: &HirStmt) -> InferResult {
        match &stmt.kind {
            HirStmtKind::Ret(ret) => {
                let Some(fn_ctx) = &self.fn_ctx else {
                    bug!("fn ctx must exist when entering a function")
                };

                let TyKind::Fn(fn_ty) = self.tctx.get(fn_ctx.ty.id) else {
                    bug!("fn ty must be the ty of the function")
                };

                // TODO: improve fn tys for precise diagnostics
                let fn_body = fn_ctx.fn_body;
                let ret_ty = fn_ty.ret_ty;

                let val_ty = if let Some(val) = ret.value {
                    Ty::new(self.infer_expr(&val)?, val.span)
                } else {
                    Ty::new(self.tctx.make_unit_ty(), ret.span)
                };

                self.check(&Ty::new(ret_ty, Span::default()), &val_ty)
                    .map(|diag| self.dctx.add(diag));
                self.tctx
                    .attach_to_hir(fn_body, Ty::new(ret_ty, Span::default()));

                Ok(())
            }

            HirStmtKind::Pass(pass) => {
                let (ty_id, span) = if let Some(value) = pass.value {
                    (self.infer_expr(&value)?, value.span)
                } else {
                    (self.tctx.make_unit_ty(), pass.span)
                };

                self.tctx.attach_to_hir(stmt.id, Ty::new(ty_id, span));
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
