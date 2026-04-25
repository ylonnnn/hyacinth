use std::collections::{HashMap, HashSet};

use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirId, HirTable,
    def::{DefId, DefinitionTable},
    item::HirPetal,
};
use hycc_ty::{
    context::{TyCtx, TyId},
    ty::{InferKind, Ty, TyKind},
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
    }
}
