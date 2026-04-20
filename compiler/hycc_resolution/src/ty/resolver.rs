use std::collections::HashMap;

use hycc_hir::{
    HirId,
    def::{DefId, DefinitionTable},
};

use crate::diag::ResolverDiagCtx;

#[derive(Debug, Clone)]
pub struct TyResolver<'d> {
    pub dctx: ResolverDiagCtx,

    pub(crate) definitions: &'d DefinitionTable,
    pub(crate) resolved: HashMap<HirId, DefId>,
}

impl<'d> TyResolver<'d> {
    pub fn new(definitions: &'d DefinitionTable, resolved: HashMap<HirId, DefId>) -> Self {
        Self {
            dctx: ResolverDiagCtx::new(),
            definitions,
            resolved,
        }
    }
}
