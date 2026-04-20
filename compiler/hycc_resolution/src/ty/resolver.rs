use std::collections::HashMap;

use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirId,
    def::{BuiltinKind, DefId, DefKind, DefinitionTable},
    item::HirPetal,
};
use hycc_ty::context::{TyCtx, TyId};

use crate::{ResolveResult, diag::ResolverDiagCtx};

#[derive(Debug)]
pub struct TyResolver<'d, 'r> {
    pub dctx: ResolverDiagCtx,
    pub tctx: TyCtx,

    pub(crate) definitions: &'d DefinitionTable,
    pub(crate) resolved: &'r HashMap<HirId, DefId>,
}

impl<'d, 'r> TyResolver<'d, 'r> {
    pub fn new(
        tctx: TyCtx,
        definitions: &'d DefinitionTable,
        resolved: &'r HashMap<HirId, DefId>,
    ) -> Self {
        Self {
            dctx: ResolverDiagCtx::new(),
            tctx,

            definitions,
            resolved,
        }
    }

    pub fn resolve(&mut self, tree: &HirPetal) {
        for item in &tree.items {
            if let Err(Some(diag)) = self.resolve_item(&item) {
                self.dctx.add(diag);
            }
        }
    }

    pub(crate) fn def_to_ty(&mut self, def_id: DefId) -> ResolveResult<TyId> {
        let def = self.definitions.get(def_id);
        let ty_id = match &def.kind {
            DefKind::Builtin(BuiltinKind::Ty(_)) | DefKind::Struct(_) => {
                self.tctx.get_ty_of_def(def_id).unwrap()
            }

            DefKind::Petal => todo!("throw error: cannot resolve petal as type"),

            _ => unreachable!(),
        };

        Ok(ty_id)
    }
}
