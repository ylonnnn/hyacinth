use std::collections::HashMap;

use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirId, HirNode, HirTable,
    def::{Binding, BuiltinKind, BuiltinTyKind, DefId, DefKind, DefSpace, DefinitionTable},
    item::{HirItem, HirItemKind},
    scope::ScopeCtx,
};
use hycc_span::Span;
use hycc_symbol::Symbol;
use hycc_ty::{
    context::{TyCtx, TyId},
    ty::InferKind,
};
use hycc_util::bug;

use crate::{
    ResolveResult,
    diag::{ResolverDiag, ResolverDiagCtx, ResolverDiagErrorKind},
};

#[derive(Debug)]
pub struct TyResolver<'t, 'd, 's, 'h> {
    pub dctx: ResolverDiagCtx,
    pub tctx: &'t mut TyCtx,
    pub definitions: &'d mut DefinitionTable,
    pub scope_ctx: &'s mut ScopeCtx,
    pub hir_table: &'h HirTable<'h>,
}

impl<'t, 'd, 's, 'h> TyResolver<'t, 'd, 's, 'h> {
    pub fn new(
        tctx: &'t mut TyCtx,
        definitions: &'d mut DefinitionTable,
        scope_ctx: &'s mut ScopeCtx,
        hir_table: &'h HirTable<'h>,
    ) -> Self {
        Self {
            dctx: ResolverDiagCtx::new(),
            tctx,
            definitions,
            scope_ctx,
            hir_table,
        }
    }

    pub(crate) fn def_to_ty(&mut self, def_id: DefId, span: Span) -> ResolveResult<TyId> {
        let def = self.definitions.get(def_id);
        let ty_id = match &def.kind {
            DefKind::Builtin(BuiltinKind::Ty(kind)) => match kind {
                BuiltinTyKind::Infer => self.tctx.make_inferred_ty(InferKind::Any),
                _ => self.tctx.get_ty_of_def(def_id).unwrap().id,
            },

            DefKind::Petal => Err(Some(ResolverDiag::error(
                span,
                ResolverDiagErrorKind::InvalidPetalResolution(def.name, def_id),
            )))?,

            DefKind::Fn(fn_def) => {
                let HirNode::Item(item) = self.hir_table.get(def.hir_id) else {
                    unreachable!()
                };

                self.resolve_fn(&item)?
            }

            _ => self.tctx.expect_hir_ty_id(def.hir_id),
        };

        Ok(ty_id)
    }
    pub fn resolve(&mut self, tree: &HirItem) {
        let HirItemKind::Petal(tree) = &tree.kind else {
            bug!("invalid type resolution! type resolution must start at the tree (a petal)")
        };

        for item in &tree.items {
            if let Err(Some(diag)) = self.resolve_item(&item) {
                self.dctx.add(diag);
            }
        }
    }
}
