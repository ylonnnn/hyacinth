use std::collections::HashSet;

use hycc_const::table::ConstTable;
use hycc_diagnostic::diagnostic::{DiagCtx, Diagnostics};
use hycc_hir::{
    HirTable,
    def::DefinitionTable,
    item::{HirItem, HirItemKind},
    petal::PetalCtx,
};
use hycc_resolve::{InstantiateIdent, ResolveExpr, ResolveIdentArgs, ResolveTy};
use hycc_span::Span;
use hycc_ty::{
    context::{TyCtx, TyId, TyVarId},
    ty::{InferKind, IntTy, Ty, TyKind},
};
use hycc_util::bug;

use crate::{
    diag::{InferDiag, InferDiagCtx, InferDiagErrorKind},
    fn_ctx::FnCtx,
};

#[derive(Debug)]
pub struct TyInferer<'i, 'h> {
    pub dctx: InferDiagCtx<'i>,
    pub(crate) fn_ctx: Option<FnCtx>,

    pub tctx: &'i mut TyCtx,
    pub definitions: &'i mut DefinitionTable,
    pub(crate) const_table: &'i ConstTable,
    pub(crate) hir_table: &'i HirTable<'h>,
    pub(crate) petal_ctx: &'i PetalCtx,
}

impl<'i, 'h> TyInferer<'i, 'h> {
    pub fn new(
        dctx: &'i mut DiagCtx,
        tctx: &'i mut TyCtx,
        definitions: &'i mut DefinitionTable,
        const_table: &'i ConstTable,
        hir_table: &'i HirTable<'h>,
        petal_ctx: &'i PetalCtx,
    ) -> Self {
        Self {
            dctx: InferDiagCtx::new(dctx),
            fn_ctx: None,

            tctx,
            definitions,
            const_table,
            hir_table,
            petal_ctx,
        }
    }

    pub fn compatible(&mut self, expected: TyId, received: TyId) -> bool {
        self.tctx.unify_ty(expected, received)
    }

    pub fn check(&mut self, expected: &Ty, received: &Ty) -> Option<InferDiag> {
        if self.compatible(expected.id, received.id) {
            None
        } else {
            Some(InferDiag::error(
                received.span,
                InferDiagErrorKind::TypeMismatch {
                    expectation_span: expected.span,
                    expected: self.tctx.resolve_ty(expected.id),
                    received: self.tctx.resolve_ty(received.id),
                },
            ))
        }
    }

    pub fn use_fn_ctx<F, U>(&mut self, ctx: FnCtx, mut handler: F) -> U
    where
        F: FnMut(&mut Self) -> U,
    {
        let prev_ctx = self.fn_ctx.take();
        self.fn_ctx.replace(ctx);

        let data = handler(self);
        self.fn_ctx = prev_ctx;

        data
    }

    pub fn infer(&mut self, tree: &HirItem) {
        let petal = tree.expect_petal();
        self.infer_petal(&petal);

        let err = *self.dctx.error_flag();
        self.analyze_unresolved(!err);
    }

    fn analyze_unresolved(&mut self, emit_err: bool) {
        let mut tys = self
            .tctx
            .hir_tys()
            .into_iter()
            .map(|(hir_id, ty)| (hir_id, ty.span))
            .collect::<Vec<_>>();

        tys.sort_by(|(_, a_span), (_, b_span)| {
            a_span
                .offset
                .cmp(&b_span.offset)
                .then_with(|| b_span.len.cmp(&a_span.len))
        });

        let mut checked = HashSet::<TyVarId>::new();

        let default_int_ty_id = self.tctx.make_int_ty(IntTy::Fixed(32, true));
        let default_float_ty_id = self.tctx.make_float_ty(32);

        for (hir_id, _) in tys {
            let ty = self.tctx.expect_hir_ty(hir_id);
            let span = ty.span;
            let ty_id = self.tctx.resolve_ty(ty.id);

            let mut unresolved_infer_tys = Vec::new();
            self.tctx.unresolved_infer(ty_id, &mut unresolved_infer_tys);

            for infer_ty in unresolved_infer_tys {
                let TyKind::Infer(var_id, kind) = self.tctx.get(infer_ty) else {
                    continue;
                };

                match &kind {
                    InferKind::Int => self.tctx.unify_ty(ty_id, default_int_ty_id),
                    InferKind::Float => self.tctx.unify_ty(ty_id, default_float_ty_id),
                    _ => {
                        if checked.insert(*var_id) && emit_err {
                            self.dctx.error(
                                span,
                                InferDiagErrorKind::UnresolvedTy(Ty::new(ty_id, span)),
                            );
                        }

                        continue;
                    }
                };
            }
        }
    }
}

impl<'i, 'h> ResolveTy<InferDiag> for TyInferer<'i, 'h> {
    fn resolve_ty(&mut self, ty: &hycc_hir::ty::HirTy) -> Result<TyId, InferDiag> {
        Ok(self.tctx.expect_hir_ty_id(ty.id))
    }
}

impl<'i, 'h> ResolveIdentArgs<TyId, InferDiag> for TyInferer<'i, 'h> {}

impl<'i, 'h> InstantiateIdent<TyId, InferDiag> for TyInferer<'i, 'h> {
    fn definitions(&self) -> &DefinitionTable {
        &self.definitions
    }

    fn tctx(&mut self) -> &mut TyCtx {
        &mut self.tctx
    }

    fn def_ty(
        &mut self,
        def_id: hycc_hir::def::DefId,
        _span: hycc_span::Span,
    ) -> Result<TyId, InferDiag> {
        let def = self.definitions.get(def_id);
        Ok(self.tctx.expect_hir_ty_id(def.hir_id))
    }

    fn generic_arg_arity_mismatch_error(
        &self,
        span: Span,
        expected: u8,
        received: u8,
    ) -> InferDiag {
        InferDiag::error(
            span,
            InferDiagErrorKind::GenericArgumentArityMismatch(
                ((expected as u16) << u8::BITS) | received as u16,
            ),
        )
    }
}
