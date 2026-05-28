use hycc_diagnostic::DiagnosticContext;
use hycc_hir::item::{HirItem, HirItemKind, HirPetal, HirStruct};
use hycc_span::Span;
use hycc_ty::ty::{InferKind, Ty, TyKind};
use hycc_util::bug;

use crate::{
    fn_ctx::FnCtx,
    inferer::{InferResult, TyInferer},
};

impl<'t, 'd, 'c, 'h> TyInferer<'t, 'd, 'c, 'h> {
    pub(crate) fn infer_item(&mut self, item: &HirItem) -> InferResult {
        match &item.kind {
            HirItemKind::Refer(_) => Ok(()),
            HirItemKind::Petal(petal) => self.infer_petal(&petal),
            HirItemKind::Struct(strct) => self.infer_struct(&strct),
            HirItemKind::Fn(_) => self.infer_fn(&item),
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

    pub(crate) fn infer_fn(&mut self, fn_item: &HirItem) -> InferResult {
        let HirItemKind::Fn(func) = &fn_item.kind else {
            unreachable!()
        };

        let Some(fn_ty) = self.tctx.get_ty_of_hir(fn_item.id).cloned() else {
            bug!("fn hir {:?} does not have an attached ty", fn_item.id)
        };

        let fn_ty_id = fn_ty.id;

        self.use_fn_ctx(FnCtx::new(fn_ty, func.body.id), |s| -> InferResult {
            let TyKind::Fn(fn_ty) = s.tctx.get(fn_ty_id) else {
                return Ok(());
            };

            let ret_ty = Ty::new(
                fn_ty.ret_ty,
                func.ret_ty.map(|ty| ty.span).unwrap_or(Span::default()),
            );
            s.tctx.attach_to_hir(func.body.id, ret_ty.clone());

            let block_ty_id = s.infer_block(&func.body)?;
            s.check(&ret_ty, &Ty::new(block_ty_id, func.body.span))
                .map(|diag| s.dctx.add(diag));

            Ok(())
        })
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
