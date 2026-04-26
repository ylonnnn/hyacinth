use std::collections::{HashMap, HashSet};

use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirId, HirTable,
    def::{DefId, DefinitionTable},
    item::HirPetal,
};
use hycc_ty::{
    context::{TyCtx, TyId},
    ty::Ty,
};

use crate::diag::{InferDiag, InferDiagCtx, InferDiagErrorKind};

#[derive(Debug)]
pub struct TyInferer<'t, 'd, 'r, 'h> {
    pub dctx: InferDiagCtx,
    pub tctx: &'t mut TyCtx,

    pub(crate) definitions: &'d DefinitionTable,
    pub(crate) resolved: &'r HashMap<HirId, DefId>,
    pub(crate) hir_table: &'h HirTable<'h>,
}

pub type InferResult<T = (), E = Option<InferDiag>> = Result<T, E>;

impl<'t, 'd, 'r, 'h> TyInferer<'t, 'd, 'r, 'h> {
    pub fn new(
        tctx: &'t mut TyCtx,
        definitions: &'d DefinitionTable,
        resolved: &'r HashMap<HirId, DefId>,
        hir_table: &'h HirTable<'h>,
    ) -> Self {
        Self {
            dctx: InferDiagCtx::new(),
            tctx,

            definitions,
            resolved,
            hir_table,
        }
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
