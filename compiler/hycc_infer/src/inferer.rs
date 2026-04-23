use std::collections::HashMap;

use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirId,
    def::{DefId, DefinitionTable},
    item::HirPetal,
};
use hycc_ty::context::TyCtx;

use crate::diag::{InferDiag, InferDiagCtx};

#[derive(Debug)]
pub struct TyInferer<'t, 'd, 'r> {
    pub dctx: InferDiagCtx,
    pub tctx: &'t mut TyCtx,

    pub(crate) definitions: &'d DefinitionTable,
    pub(crate) resolved: &'r HashMap<HirId, DefId>,
}

pub type InferResult<T = (), E = Option<InferDiag>> = Result<T, E>;

impl<'t, 'd, 'r> TyInferer<'t, 'd, 'r> {
    pub fn new(
        tctx: &'t mut TyCtx,
        definitions: &'d DefinitionTable,
        resolved: &'r HashMap<HirId, DefId>,
    ) -> Self {
        Self {
            dctx: InferDiagCtx::new(),
            tctx,

            definitions,
            resolved,
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
