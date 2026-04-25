use hycc_diagnostic::DiagnosticContext;
use hycc_hir::item::{HirFn, HirItem, HirItemKind, HirPetal, HirStruct};
use hycc_span::Span;
use hycc_ty::context::TyId;
use hycc_util::bug;

use crate::{
    diag::{InferDiag, InferDiagErrorKind},
    inferer::{InferResult, TyInferer},
};

impl<'t, 'd, 'r, 'h> TyInferer<'t, 'd, 'r, 'h> {
    pub(crate) fn infer_item(&mut self, item: &HirItem) -> InferResult {
        match &item.kind {
            HirItemKind::Petal(petal) => self.infer_petal(&petal),
            HirItemKind::Struct(strct) => self.infer_struct(&strct),
            HirItemKind::Fn(func) => self.infer_fn(&func),
            HirItemKind::VarDecl(_) => self.infer_var_decl(&item),
        }
    }

    pub(crate) fn infer_petal(&mut self, petal: &HirPetal) -> InferResult {
        for item in &petal.items {
            if let Err(Some(diag)) = self.infer_item(&item) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }

    pub(crate) fn infer_struct(&mut self, strct: &HirStruct) -> InferResult {
        // TODO
        Ok(())
    }

    pub(crate) fn infer_fn(&mut self, func: &HirFn) -> InferResult {
        // TODO: allow infer_block to return its equivalent TyId
        if let Err(Some(diag)) = self.infer_block(&func.body) {
            self.dctx.add(diag);
        }

        Ok(())
    }

    pub(crate) fn infer_var_decl(&mut self, var_decl: &HirItem) -> InferResult {
        let HirItemKind::VarDecl(decl) = &var_decl.kind else {
            unreachable!()
        };

        let ty = decl.ty.map(|ty| {
            let Some(ty_id) = self.tctx.get_ty_of_hir(ty.id) else {
                bug!("var decl ty hir is not attached to a TyId: {:?}", ty.id)
            };

            ty_id
        });

        if let Some(expr) = decl.val {
            let expr_ty = self.infer_expr(&expr)?;
            let var_ty: TyId;

            if let Some(ty) = ty {
                if !self.tctx.unify_ty(ty, expr_ty) {
                    return Err(Some(InferDiag::error(
                        expr.span,
                        InferDiagErrorKind::TypeMismatch {
                            ann_span: decl.ty.unwrap().span,
                            expected: ty,
                            received: expr_ty,
                        },
                    )));
                }

                var_ty = ty;
            } else {
                var_ty = self.tctx.resolve_ty(expr_ty);
            }

            self.tctx.attach_to_hir(var_decl.id, var_ty);
        }

        Ok(())
    }
}
