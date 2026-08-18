use hycc_diagnostic::diagnostic::{Diagnostics, FromResultEmitter};
use hycc_hir::item::{HirExtend, HirItem, HirItemKind, HirPetal, HirStruct};
use hycc_span::Span;
use hycc_ty::ty::{InferKind, Ty, TyKind};
use hycc_util::bug;

use crate::{diag::InferResult, fn_ctx::FnCtx, inferer::TyInferer};

impl<'i, 'h> TyInferer<'i, 'h> {
    pub(crate) fn infer_item(&mut self, item: &HirItem) -> InferResult {
        match &item.kind {
            HirItemKind::Refer(_) => Ok(()),
            HirItemKind::Petal(petal) => self.infer_petal(&petal),
            HirItemKind::Proto(proto) => todo!("infer proto"),
            HirItemKind::Extend(extend) => self.infer_extend(&extend),
            HirItemKind::Struct(strct) => Ok(()),
            HirItemKind::Fn(_) => self.infer_fn(&item),
            HirItemKind::VarDecl(_) => self.infer_var_decl(&item),
        }
    }

    pub(crate) fn infer_petal(&mut self, petal: &HirPetal) -> InferResult {
        petal
            .items
            .iter()
            .for_each(|item| self.infer_item(&item).emit_discard(&mut self.dctx));

        Ok(())
    }

    pub(crate) fn infer_extend(&mut self, extend: &HirExtend) -> InferResult {
        // Infer items of the extension
        extend
            .items
            .iter()
            .for_each(|item| self.infer_item(&item).emit_discard(&mut self.dctx));

        Ok(())
    }

    pub(crate) fn infer_fn(&mut self, item: &HirItem) -> InferResult {
        let func = &item.expect_fn();

        let Some(fn_ty) = self.tctx.get_hir_ty(item.id).cloned() else {
            bug!("fn hir {:?} does not have an attached ty", item.id)
        };

        let fn_ty_id = fn_ty.id;

        self.use_fn_ctx(FnCtx::new(fn_ty, func.body.id), |s| -> InferResult {
            let TyKind::Fn(fn_ty, _) = s.tctx.get(fn_ty_id) else {
                return Ok(());
            };

            let ret_ty = Ty::new(
                fn_ty.ret_ty,
                func.sig.ret_ty.map(|ty| ty.span).unwrap_or(Span::default()),
            );
            s.tctx.attach_to_hir(func.body.id, ret_ty.clone());

            let block_ty_id = s.infer_block(&func.body)?;
            s.check(&ret_ty, &Ty::new(block_ty_id, func.body.span))
                .map(|diag| s.dctx.add(diag));

            s.analyze_unresolved();

            Ok(())
        })
    }

    pub(crate) fn infer_var_decl(&mut self, var_decl: &HirItem) -> InferResult {
        let HirItemKind::VarDecl(decl) = &var_decl.kind else {
            unreachable!()
        };

        let ty = decl
            .ty
            .and_then(|ty| self.tctx.get_hir_ty(ty.id).cloned())
            .unwrap_or_else(|| {
                let default_ty_id = self
                    .tctx
                    .get_hir_ty_id(var_decl.id)
                    .unwrap_or_else(|| self.tctx.make_inferred_ty(decl.span, InferKind::Any));
                Ty::new(default_ty_id, decl.span)
            });

        if let Some(expr) = decl.val {
            let expr_ty = self.infer_expr(&expr)?;

            self.check(&ty, &Ty::new(expr_ty, expr.span))
                .map(|diag| self.dctx.add(diag));

            if let Some(def) = self.definitions.get_def(var_decl.id) {
                self.tctx.attach_to_hir(def.hir_id, ty);
            }
        }

        Ok(())
    }
}
