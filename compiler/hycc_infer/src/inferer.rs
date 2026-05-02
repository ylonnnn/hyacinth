use std::collections::{HashMap, HashSet};

use hycc_const::table::ConstTable;
use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirId,
    def::{DefId, DefinitionTable},
    item::HirPetal,
};
use hycc_ty::{
    context::{TyCtx, TyId},
    ty::Ty,
};

use crate::{
    diag::{InferDiag, InferDiagCtx, InferDiagErrorKind},
    fn_ctx::FnCtx,
};

#[derive(Debug)]
pub struct TyInferer<'t, 'd, 'r, 'c> {
    pub dctx: InferDiagCtx,
    pub tctx: &'t mut TyCtx,

    pub(crate) definitions: &'d DefinitionTable,
    pub(crate) resolved: &'r HashMap<HirId, DefId>,
    pub(crate) const_table: &'c ConstTable,

    pub(crate) fn_ctx: Option<FnCtx>,
}

pub type InferResult<T = (), E = Option<InferDiag>> = Result<T, E>;

impl<'t, 'd, 'r, 'c> TyInferer<'t, 'd, 'r, 'c> {
    pub fn new(
        tctx: &'t mut TyCtx,
        definitions: &'d DefinitionTable,
        resolved: &'r HashMap<HirId, DefId>,
        const_table: &'c ConstTable,
    ) -> Self {
        Self {
            dctx: InferDiagCtx::new(),
            tctx,

            definitions,
            resolved,
            const_table,

            fn_ctx: None,
        }
    }

    pub fn check(&mut self, expected: &Ty, received: &Ty) -> Option<InferDiag> {
        if self.tctx.unify_ty(expected.id, received.id) {
            None
        } else {
            Some(InferDiag::error(
                received.span,
                InferDiagErrorKind::TypeMismatch {
                    ann_span: expected.span,
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

    pub fn infer(&mut self, tree: &HirPetal) {
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
            let Some(ty) = self.tctx.get_ty_of_hir(hir_id) else {
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
