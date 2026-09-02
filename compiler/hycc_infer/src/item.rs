use hycc_diagnostic::diagnostic::{Diagnostics, FromResultEmitter};
use hycc_hir::{
    HirId,
    item::{
        HirExtend, HirFnSig, HirIntf, HirIntfItem, HirItem, HirItemKind, HirPetal, HirStruct,
        HirVarSig,
    },
};
use hycc_span::Span;
use hycc_ty::{
    ctx::{TyId, TyResState},
    ty::{InferKind, Ty, TyKind},
};
use hycc_util::{bug, ternary};

use crate::{
    diag::{InferDiagErrorKind, InferResult},
    fn_ctx::FnCtx,
    inferer::TyInferer,
};

impl<'i, 'h> TyInferer<'i, 'h> {
    pub(crate) fn check_item(&mut self, item: &HirItem) -> InferResult {
        match &item.kind {
            HirItemKind::Refer(_) => Ok(()),
            HirItemKind::Petal(petal) => self.check_petal(&petal),
            HirItemKind::Intf(intf) => self.check_intf(&intf),
            HirItemKind::Extend(extend) => self.check_extend(&extend),
            HirItemKind::Struct(strct) => Ok(()),
            HirItemKind::FnDecl(sig) => self.check_fn_sig(item.id, &sig),
            HirItemKind::Fn(_) => self.check_fn(&item),
            HirItemKind::VarDecl(sig) => self.check_var_sig(item.id, &sig),
            HirItemKind::VarDef(_) => self.check_var_def(&item),
        }
    }

    pub(crate) fn infer_item(&mut self, item: &HirItem) -> InferResult {
        match &item.kind {
            HirItemKind::Refer(_) => Ok(()),
            HirItemKind::Petal(petal) => self.infer_petal(&petal),
            HirItemKind::Intf(intf) => self.infer_intf(&intf),
            HirItemKind::Extend(extend) => self.infer_extend(&extend),
            HirItemKind::Struct(strct) => Ok(()),
            HirItemKind::FnDecl(_) => Ok(()),
            HirItemKind::Fn(_) => self.infer_fn(&item),
            HirItemKind::VarDecl(_) => Ok(()),
            HirItemKind::VarDef(_) => self.infer_var_def(&item),
        }
    }

    // TODO: potentially merge check and inference logic
    pub(crate) fn check_petal(&mut self, petal: &HirPetal) -> InferResult {
        petal
            .items
            .iter()
            .for_each(|item| self.check_item(&item).emit_discard(&mut self.dctx));

        Ok(())
    }

    pub(crate) fn infer_petal(&mut self, petal: &HirPetal) -> InferResult {
        let petals = petal
            .path()
            .map(|path| {
                path.segments
                    .iter()
                    .map(|segment| {
                        self.petal_ctx
                            .expect_def_petal_id(self.definitions.expect_def_id(segment.id))
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec![self.petal_ctx.root_petal_id()]);

        petals
            .iter()
            .for_each(|petal_id| self.petal_ctx.push(*petal_id));

        petal
            .items
            .iter()
            .for_each(|item| self.infer_item(&item).emit_discard(&mut self.dctx));

        (0..petals.len()).for_each(|_| self.petal_ctx.pop());

        Ok(())
    }

    pub(crate) fn check_intf(&mut self, intf: &HirIntf) -> InferResult {
        intf.items
            .iter()
            .for_each(|item| self.check_intf_item(&item).emit_discard(&mut self.dctx));

        Ok(())
    }

    pub(crate) fn check_intf_item(&mut self, item: &HirIntfItem) -> InferResult {
        match &item {
            HirIntfItem::Fn(item) | HirIntfItem::Var(item) => self.check_item(&item),
        }
    }

    pub(crate) fn infer_intf(&mut self, intf: &HirIntf) -> InferResult {
        intf.items
            .iter()
            .for_each(|item| self.infer_intf_item(&item).emit_discard(&mut self.dctx));

        Ok(())
    }

    pub(crate) fn infer_intf_item(&mut self, item: &HirIntfItem) -> InferResult {
        match &item {
            HirIntfItem::Fn(item) | HirIntfItem::Var(item) => self.infer_item(&item),
        }
    }

    // TODO: potentially merge check and inference logic
    pub(crate) fn check_extend(&mut self, extend: &HirExtend) -> InferResult {
        // Check items of the extension
        extend
            .items
            .iter()
            .for_each(|item| self.check_item(&item).emit_discard(&mut self.dctx));

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

    pub(crate) fn check_fn_sig(&mut self, hir_id: HirId, sig: &HirFnSig) -> InferResult {
        if matches!(
            self.tctx.expect_hir_res_state(hir_id),
            TyResState::Inferred(_) | TyResState::Resolved(_)
        ) {
            return Ok(());
        }

        let fn_ty = self.tctx.expect_hir_ty(hir_id).clone();
        let TyKind::Fn(fn_ty, _) = self.tctx.get(fn_ty.id) else {
            unreachable!()
        };

        self.dctx.error(
            sig.ret_ty.map_or_else(|| sig.span, |ret_ty| ret_ty.span),
            InferDiagErrorKind::InvalidInference(fn_ty.ret_ty),
        );

        Ok(())
    }

    pub(crate) fn check_fn(&mut self, item: &HirItem) -> InferResult {
        let func = &item.expect_fn();
        self.check_fn_sig(item.id, &func.sig).emit(&mut self.dctx);

        let fn_ty = self.tctx.expect_hir_ty(item.id).clone();
        ternary!(
            matches!(
                self.tctx.expect_hir_res_state(item.id),
                TyResState::Resolved(_)
            ),
            self.check_block(&func.body),
            self.infer_fn(&item)
        )
    }

    pub(crate) fn infer_fn(&mut self, item: &HirItem) -> InferResult {
        if matches!(
            self.tctx.expect_hir_res_state(item.id),
            TyResState::Inferred(_)
        ) {
            return Ok(());
        }

        let func = &item.expect_fn();
        let fn_ty = self.tctx.expect_hir_ty(item.id).clone();

        self.use_fn_ctx(FnCtx::new(fn_ty, func.body.id), |s| -> InferResult {
            let ctx = s.fn_ctx.as_ref().unwrap();
            let TyKind::Fn(fn_ty, _) = s.tctx.get(ctx.ty.id) else {
                unreachable!()
            };

            let (fn_ty_id, ret_ty_id) = (ctx.ty.id, fn_ty.ret_ty);

            let ret_ty = Ty::new(
                ret_ty_id,
                func.sig.ret_ty.map(|ty| ty.span).unwrap_or(Span::default()),
            );

            s.tctx.update_hir_res_state(item.id, TyResState::Resolving);
            s.tctx.attach_to_hir(func.body.id, ret_ty.clone());

            let block_ty_id = s.infer_block(&func.body)?;
            s.check(&ret_ty, &Ty::new(block_ty_id, func.body.span))
                .emit(&mut s.dctx);

            let resolved_fn_ty_id = s.tctx.resolve_ty(fn_ty_id);
            s.tctx
                .update_hir_res_state(item.id, TyResState::Inferred(resolved_fn_ty_id));

            s.analyze_unresolved();

            Ok(())
        })
    }

    pub(crate) fn check_var_sig(&mut self, hir_id: HirId, sig: &HirVarSig) -> InferResult {
        if matches!(
            self.tctx.expect_hir_res_state(hir_id),
            TyResState::Inferred(_) | TyResState::Resolved(_)
        ) {
            return Ok(());
        }

        let ty_id = self.tctx.expect_hir_ty_id(hir_id);
        self.dctx.error(
            sig.ty.map_or_else(|| sig.span, |ty| ty.span),
            InferDiagErrorKind::InvalidInference(ty_id),
        );

        Ok(())
    }

    pub(crate) fn check_var_def(&mut self, item: &HirItem) -> InferResult {
        if matches!(
            self.tctx.expect_hir_res_state(item.id),
            TyResState::Inferred(_)
        ) {
            return Ok(());
        }

        let def = item.expect_var_def();
        self.check_var_sig(item.id, &def.sig).emit(&mut self.dctx);

        ternary!(
            !matches!(
                self.tctx.expect_hir_res_state(item.id),
                TyResState::Resolved(_)
            ),
            self.infer_var_def(&item),
            // Recursively check each item of the body without
            // performing unnecessary expression type inference
            def.val
                .as_ref()
                .map_or_else(|| Ok(()), |val| self.check_expr(&val))
        )
    }

    pub(crate) fn infer_var_def(&mut self, item: &HirItem) -> InferResult {
        if matches!(
            self.tctx.expect_hir_res_state(item.id),
            TyResState::Inferred(_)
        ) {
            return Ok(());
        }

        let def = item.expect_var_def();
        let ty =
            def.sig
                .ty
                .and_then(|ty| self.tctx.get_hir_ty(ty.id).cloned())
                .unwrap_or_else(|| {
                    let default_ty_id = self.tctx.get_hir_ty_id(item.id).unwrap_or_else(|| {
                        self.tctx.make_inferred_ty(def.sig.span, InferKind::Any)
                    });
                    Ty::new(default_ty_id, def.sig.span)
                });

        if let Some(expr) = def.val {
            self.tctx
                .update_hir_res_state(item.id, TyResState::Resolving);
            self.infer_expr(&expr, Some(ty)).emit(&mut self.dctx);

            self.definitions
                .get_def(item.id)
                .map(|def| self.tctx.attach_to_hir(def.hir_id, ty));
        }

        Ok(())
    }
}
