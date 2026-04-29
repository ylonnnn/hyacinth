use hycc_diagnostic::DiagnosticContext;
use hycc_hir::item::{HirFn, HirItem, HirItemKind, HirPetal, HirStruct};
use hycc_span::Span;
use hycc_ty::ty::{InferKind, Ty};
use hycc_util::bug;

use crate::{
    diag::{InferDiag, InferDiagErrorKind},
    inferer::{InferResult, TyInferer},
};

impl<'t, 'd, 'r> TyInferer<'t, 'd, 'r> {
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
            let Some(ty) = self.tctx.get_ty_of_hir(ty.id).cloned() else {
                bug!("var decl ty hir is not attached to a Ty: {:?}", ty.id)
            };

            ty
        });

        if let Some(expr) = decl.val {
            let expr_ty = self.infer_expr(&expr)?;
            let ty = ty.unwrap_or(Ty::new(
                self.tctx.make_inferred_ty(InferKind::Any),
                Span::default(),
            ));

            self.check(&ty, &Ty::new(expr_ty, expr.span))
                .map(|diag| self.dctx.add(diag));

            self.tctx.attach_to_hir(var_decl.id, ty);
        }

        Ok(())
    }
}
