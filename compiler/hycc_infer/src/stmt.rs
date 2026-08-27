use hycc_diagnostic::diagnostic::{Diagnostics, FromResultEmitter};
use hycc_hir::{
    HirNode,
    stmt::{HirStmt, HirStmtKind},
};
use hycc_resolve::InstantiateIdent;
use hycc_span::Span;
use hycc_ty::ty::{Ty, TyKind};
use hycc_util::bug;

use crate::{diag::InferResult, inferer::TyInferer};

impl<'i, 'h> TyInferer<'i, 'h> {
    pub(crate) fn check_stmt(&mut self, stmt: &HirStmt) -> InferResult {
        match &stmt.kind {
            HirStmtKind::Ret(ret) => ret
                .value
                .as_ref()
                .map_or(Ok(()), |val| self.check_expr(&val)),

            HirStmtKind::Pass(pass) => pass
                .value
                .as_ref()
                .map_or(Ok(()), |val| self.check_expr(&val)),

            HirStmtKind::Item(item) => self.check_item(&item),
            HirStmtKind::Expr(expr) => self.check_expr(&expr),
        }
    }

    pub(crate) fn infer_stmt(&mut self, stmt: &HirStmt) -> InferResult {
        let ty_id = match &stmt.kind {
            HirStmtKind::Ret(ret) => {
                let fn_ctx = self
                    .fn_ctx
                    .as_ref()
                    .expect("fn ctx must exist when entering a function");

                let TyKind::Fn(fn_ty, _) = self.tctx.get(fn_ctx.ty.id) else {
                    bug!("fn ty must be the ty of the function")
                };

                // TODO: improve diagnostics (?)
                let fn_body = fn_ctx.fn_body;
                let ret_ty = Ty::new(
                    fn_ty.ret_ty,
                    fn_ty
                        .def_id
                        .and_then(|def_id| {
                            let def = self.definitions.get(def_id).kind.expect_fn();
                            def.ret_ty.and_then(|ret_ty| {
                                let HirNode::Ty(ty) = self.hir_table.get(ret_ty) else {
                                    return None;
                                };

                                Some(ty.span)
                            })
                        })
                        .unwrap_or_else(|| Span::default()),
                );

                ret.value
                    .map(|val| self.infer_expr(&val, Some(ret_ty)).emit(&mut self.dctx));

                Ok(Some(ret_ty.id))
            }

            HirStmtKind::Pass(pass) => {
                let (ty_id, span) = if let Some(value) = pass.value {
                    (self.infer_expr(&value, None)?, value.span)
                } else {
                    (self.tctx.make_unit_ty(), pass.span)
                };

                self.tctx.attach_to_hir(stmt.id, Ty::new(ty_id, span));
                Ok(Some(ty_id))
            }

            HirStmtKind::Item(item) => {
                self.infer_item(&item)?;
                Ok(self.tctx.get_hir_ty_id(item.id))
            }
            HirStmtKind::Expr(expr) => self.infer_expr(&expr, None).map(|ty_id| Some(ty_id)),
        }?;

        ty_id.map(|ty_id| self.tctx.attach_to_hir(stmt.id, Ty::new(ty_id, stmt.span)));
        Ok(())
    }
}
