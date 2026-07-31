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
    context::{TyCtx, TyId},
    ty::Ty,
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
                    expected: expected.id,
                    received: received.id,
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

        self.check_unresolved();
    }

    fn check_unresolved(&mut self) {
        let mut tys = self
            .tctx
            .hir_tys()
            .into_iter()
            .map(|(hir_id, ty)| (hir_id, ty.span))
            .collect::<Vec<_>>();

        tys.sort_by_key(|(_, span)| span.offset);
        let tys = tys
            .into_iter()
            .map(|(hir_id, _)| hir_id)
            .collect::<Vec<_>>();

        let mut checked = HashSet::<TyId>::new();

        for hir_id in tys {
            let Some(ty) = self.tctx.get_hir_ty(hir_id) else {
                continue;
            };

            if checked.contains(&ty.id) {
                continue;
            }

            checked.insert(ty.id);

            let span = ty.span;
            let ty_id = self.tctx.resolve_ty(ty.id);

            if !self.tctx.is_inferred(ty_id) {
                continue;
            };

            self.dctx.add(InferDiag::error(
                span,
                InferDiagErrorKind::UnresolvedTy(Ty::new(ty_id, span)),
            ));
        }
    }
}
