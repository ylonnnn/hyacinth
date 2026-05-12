use std::collections::HashMap;

use hycc_collection::{
    collector::{CollectionLevel, Collector},
    diag::CollectorDiagDataCtx,
};
use hycc_diagnostic::{DiagnosticContext, DiagnosticCtx};
use hycc_hir::{
    HirId,
    def::{DefId, DefSpace, Definition},
    item::HirPetal,
};
use hycc_scope::ScopeId;
use hycc_symbol::Symbol;

use crate::diag::{ResolverDiagCtx, ResolverDiagDataCtx};

#[derive(Debug)]
pub struct Resolver<'c> {
    pub dctx: ResolverDiagCtx,
    pub collector: &'c mut Collector,

    // The expected space to retrieve unresolve paths from.
    pub(crate) expected_space: Option<DefSpace>,
}

impl<'c> Resolver<'c> {
    pub fn new(collector: &'c mut Collector) -> Self {
        Self {
            dctx: ResolverDiagCtx::new(),
            collector,

            expected_space: None,
        }
    }

    pub fn get_def_id(&self, space: DefSpace, name: Symbol) -> Option<DefId> {
        // dbg!(&self.collector.scope_ctx);
        self.collector.scope_ctx.get_def_until_root(space, name)
    }

    pub fn get_def(&self, space: DefSpace, name: Symbol) -> Option<&Definition> {
        let def_id = self.get_def_id(space, name)?;
        Some(self.collector.definitions.get(def_id))
    }

    pub fn enter_scope<F, U>(&mut self, scope_id: ScopeId, mut handler: F) -> U
    where
        F: FnMut(&mut Self) -> U,
    {
        let prev_level = self.collector.node_level;
        self.collector.node_level = CollectionLevel::Local;
        self.collector.scope_ctx.push_id(scope_id);

        let data = handler(self);
        self.collector.scope_ctx.pop();
        self.collector.node_level = prev_level;

        data
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

    pub fn emit_dctx(
        &mut self,
        target: &mut DiagnosticCtx,
        collector_data_ctx: CollectorDiagDataCtx,
        resolver_data_ctx: ResolverDiagDataCtx,
    ) {
        self.collector.dctx.emit(target, collector_data_ctx);
        self.dctx.emit(target, resolver_data_ctx);
    }

    pub fn resolve(&mut self, tree: &HirPetal) {
        self.collector.level = CollectionLevel::Local;

        for item in &tree.items {
            if let Err(Some(diag)) = self.resolve_item(&item) {
                self.dctx.add(diag);
            }
        }
    }
}
