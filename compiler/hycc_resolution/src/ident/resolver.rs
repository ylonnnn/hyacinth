use std::collections::HashMap;

use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirId,
    def::{DefId, DefSpace, Definition, DefinitionTable},
    item::HirPetal,
};
use hycc_scope::{ScopeCtx, ScopeId};
use hycc_symbol::Symbol;

use crate::diag::{ResolverDiag, ResolverDiagCtx};

#[derive(Debug)]
pub struct Resolver<'s, 'd> {
    pub dctx: ResolverDiagCtx,

    pub(crate) scope_ctx: &'s mut ScopeCtx,
    pub(crate) definitions: &'d DefinitionTable,
    pub resolved: HashMap<HirId, DefId>,

    // The expected space to retrieve unresolve paths from.
    pub(crate) expected_space: Option<DefSpace>,
}

pub type ResolveResult<T = (), E = Option<ResolverDiag>> = Result<T, E>;

impl<'s, 'd> Resolver<'s, 'd> {
    pub fn new(scope_ctx: &'s mut ScopeCtx, definitions: &'d DefinitionTable) -> Self {
        Self {
            dctx: ResolverDiagCtx::new(),
            scope_ctx,
            definitions,
            resolved: HashMap::new(),

            expected_space: None,
        }
    }

    pub fn get_def_id(&self, space: DefSpace, name: Symbol) -> Option<DefId> {
        self.scope_ctx.get_def_until_root(space, name)
    }

    pub fn get_def(&self, space: DefSpace, name: Symbol) -> Option<&Definition> {
        let def_id = self.get_def_id(space, name)?;
        Some(self.definitions.get(def_id))
    }

    pub fn enter_scope<F>(&mut self, scope_id: ScopeId, mut handler: F)
    where
        F: FnMut(&mut Self),
    {
        self.scope_ctx.push_id(scope_id);
        handler(self);
        self.scope_ctx.pop();
    }

    pub fn expect_space<F, R>(&mut self, space: DefSpace, mut handler: F) -> R
    where
        F: FnMut(&mut Self) -> R,
    {
        let prev_space = self.expected_space;

        self.expected_space = Some(space);
        let data = handler(self);

        self.expected_space = prev_space;
        data
    }

    pub fn resolve(&mut self, tree: &HirPetal) {
        for item in &tree.items {
            if let Err(Some(diag)) = self.resolve_item(&item) {
                self.dctx.add(diag);
            }
        }
    }
}
