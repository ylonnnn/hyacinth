use std::collections::HashSet;

use hycc_const::table::ConstTable;
use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirTable,
    def::DefinitionTable,
    item::{HirItem, HirItemKind},
    petal::PetalCtx,
};
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
pub struct TyInferer<'t, 'd, 'c, 'h, 'p> {
    pub dctx: InferDiagCtx,
    pub tctx: &'t mut TyCtx,

    pub definitions: &'d mut DefinitionTable,
    pub(crate) const_table: &'c ConstTable,
    pub(crate) hir_table: &'h HirTable<'h>,
    pub(crate) petal_ctx: &'p PetalCtx,

    pub(crate) fn_ctx: Option<FnCtx>,
}

pub type InferResult<T = (), E = Option<InferDiag>> = Result<T, E>;

impl<'t, 'd, 'c, 'h, 'p> TyInferer<'t, 'd, 'c, 'h, 'p> {
    pub fn new(
        tctx: &'t mut TyCtx,
        definitions: &'d mut DefinitionTable,
        const_table: &'c ConstTable,
        hir_table: &'h HirTable<'h>,
        petal_ctx: &'p PetalCtx,
    ) -> Self {
        Self {
            dctx: InferDiagCtx::new(),
            tctx,

            definitions,
            const_table,
            hir_table,
            petal_ctx,

            fn_ctx: None,
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
        let HirItemKind::Petal(tree) = &tree.kind else {
            bug!("invalid type inference! type inference must start at the tree (a petal)")
        };

        for item in &tree.items {
            if let Err(Some(diag)) = self.infer_item(&item) {
                self.dctx.add(diag);
            }
        }

        self.analyze_unresolved(self.dctx.error_occurred());
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

            if unresolved_infer_tys.is_empty() {
                continue;
            }

            for infer_ty in unresolved_infer_tys {
                let TyKind::Infer(var_id, kind) = self.tctx.get(infer_ty) else {
                    continue;
                };

                match &kind {
                    InferKind::Int => self.tctx.unify_ty(ty_id, default_int_ty_id),
                    InferKind::Float => self.tctx.unify_ty(ty_id, default_float_ty_id),
                    _ => {
                        if checked.insert(*var_id) && emit_err {
                            self.dctx.add(InferDiag::error(
                                span,
                                InferDiagErrorKind::UnresolvedTy(Ty::new(ty_id, span)),
                            ));
                        }

                        continue;
                    }
                };
            }
        }
    }
}
