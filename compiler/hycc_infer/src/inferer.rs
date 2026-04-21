use std::collections::HashMap;

use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirId,
    def::{DefId, DefinitionTable},
    item::HirPetal,
};
use hycc_ty::context::{TyCtx, TyId};

use crate::diag::{InferDiag, InferDiagCtx};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceLevel {
    Global,
    Local,
}

impl InferenceLevel {
    pub fn is_global(&self) -> bool {
        *self == Self::Global
    }

    pub fn is_local(&self) -> bool {
        *self == Self::Local
    }
}

#[derive(Debug)]
pub struct TyInferer<'t, 'd, 'r> {
    pub dctx: InferDiagCtx,
    pub tctx: &'t mut TyCtx,

    pub(crate) definitions: &'d DefinitionTable,
    pub(crate) resolved: &'r HashMap<HirId, DefId>,

    pub(crate) level: InferenceLevel,
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

            level: InferenceLevel::Global,
        }
    }

    pub fn delve<F>(&mut self, mut handler: F)
    where
        F: FnMut(&mut Self),
    {
        let prev_level = self.level;

        self.level = InferenceLevel::Local;
        handler(self);

        self.level = prev_level;
    }

    pub fn infer(&mut self, tree: &HirPetal) {
        for item in &tree.items {
            if let Err(Some(diag)) = self.infer_item(&item) {
                self.dctx.add(diag);
            }
        }
    }
}
