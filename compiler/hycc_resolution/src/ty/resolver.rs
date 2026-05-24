use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    def::{BuiltinKind, BuiltinTyKind, DefId, DefKind, DefinitionTable},
    item::HirPetal,
};
use hycc_span::Span;
use hycc_ty::{
    context::{TyCtx, TyId},
    ty::InferKind,
};

use crate::{
    ResolveResult,
    diag::{ResolverDiag, ResolverDiagCtx, ResolverDiagErrorKind},
};

#[derive(Debug)]
pub struct TyResolver<'d> {
    pub dctx: ResolverDiagCtx,
    pub tctx: TyCtx,

    pub(crate) definitions: &'d DefinitionTable,
}

impl<'d> TyResolver<'d> {
    pub fn new(tctx: TyCtx, definitions: &'d DefinitionTable) -> Self {
        Self {
            dctx: ResolverDiagCtx::new(),
            tctx,

            definitions,
        }
    }

    pub(crate) fn def_to_ty(&mut self, def_id: DefId, span: Span) -> ResolveResult<TyId> {
        let def = self.definitions.get(def_id);
        let ty_id = match &def.kind {
            DefKind::Builtin(BuiltinKind::Ty(kind)) => match kind {
                BuiltinTyKind::Infer => self.tctx.make_inferred_ty(InferKind::Any),
                _ => self.tctx.get_ty_of_def(def_id).unwrap().id,
            },

            DefKind::Struct(_) => self.tctx.get_ty_of_hir(def.hir_id).unwrap().id,

            DefKind::Petal => Err(Some(ResolverDiag::error(
                span,
                ResolverDiagErrorKind::InvalidPetalResolution(def.name, def_id),
            )))?,

            _ => unreachable!(),
        };

        Ok(ty_id)
    }

    pub fn resolve(&mut self, tree: &HirPetal) {
        for item in &tree.items {
            if let Err(Some(diag)) = self.resolve_item(&item) {
                self.dctx.add(diag);
            }
        }
    }
}
