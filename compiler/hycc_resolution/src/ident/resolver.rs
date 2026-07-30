use hycc_collection::{
    collector::{CollectionLevel, Collector},
    diag::CollectorDiagDataCtx,
};
use hycc_diagnostic::{DiagnosticContext, DiagnosticCtx};
use hycc_hir::{
    HirTable,
    def::{Binding, DefAccessibility, DefId, DefSpace, Definition},
    item::{HirItem, HirItemKind, HirPetal},
    scope::ScopeId,
};
use hycc_span::Span;
use hycc_symbol::Symbol;
use hycc_util::bug;

use crate::{
    ResolveResult,
    diag::{ResolverDiag, ResolverDiagCtx, ResolverDiagDataCtx, ResolverDiagErrorKind},
    // ident::ctx::{Petal, ResolutionCtx},
};

#[derive(Debug, Clone, Copy)]
pub enum ResolutionCtx {
    /// The default/normal resolution context
    Default,

    /// The resolution context during `Extension`/`extend` fresolution
    Extension,
}

#[derive(Debug)]
pub struct Resolver<'c, 'i, 'h> {
    pub dctx: ResolverDiagCtx,
    pub collector: &'c mut Collector<'i>,
    pub hir_table: &'h HirTable<'h>,

    // pub(crate) resolution: ResolutionCtx,

    // The expected space to retrieve unresolve paths from.
    pub(crate) expected_space: Option<DefSpace>,

    pub(crate) curr_scope: Option<ScopeId>,
}

impl<'c, 'i, 'h> Resolver<'c, 'i, 'h> {
    pub fn new(collector: &'c mut Collector<'i>, hir_table: &'h HirTable<'h>) -> Self {
        Self {
            dctx: ResolverDiagCtx::new(),
            collector,
            hir_table,
            // resolution: ResolutionCtx::Default,
            expected_space: None,
            curr_scope: None,
        }
    }

    pub fn get_binding(
        &self,
        space: Option<DefSpace>,
        name: Symbol,
    ) -> Option<(&Binding, ScopeId)> {
        let scope_id = self
            .collector
            .petal_ctx
            .top()
            .scope_id(&self.collector.scope_ctx);

        self.collector
            .scope_ctx
            .get_def_until_scope(space, name, scope_id)
            .map(|binding| (binding, scope_id))
    }

    pub fn get_def_id(&self, space: Option<DefSpace>, name: Symbol) -> Option<(DefId, ScopeId)> {
        println!("space: {space:?}, name: {name:?}");
        dbg!(self.get_binding(space, name));
        self.get_binding(space, name)
            .map(|(binding, scope_id)| (binding.def_id, scope_id))
    }

    pub fn get_def(&self, space: Option<DefSpace>, name: Symbol) -> Option<&Definition> {
        let (def_id, _) = self.get_def_id(space, name)?;
        Some(self.collector.definitions.get(def_id))
    }

    pub fn enter_scope<F, U>(&mut self, scope_id: ScopeId, mut handler: F) -> U
    where
        F: FnMut(&mut Self) -> U,
    {
        // let prev_level = self.collector.node_level;
        // self.collector.node_level = CollectionLevel::Local;
        self.collector.scope_ctx.push_id(scope_id);

        let data = handler(self);
        self.collector.scope_ctx.pop();
        // self.collector.node_level = prev_level;

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

    // pub fn check_accessibility(
    //     &mut self,
    //     def_id: DefId,
    //     symbol: Symbol,
    //     span: Span,
    // ) -> ResolveResult {
    //     let definition = self.collector.definitions.get(def_id);
    //     match definition.accessibility {
    //         DefAccessibility::Pub(_) => Ok(()),

    //         DefAccessibility::Priv => Err(Some(ResolverDiag::error(
    //             span,
    //             ResolverDiagErrorKind::InaccessibleSymbol(symbol),
    //         )))?,
    //     }
    // }

    pub fn emit_dctx(
        &mut self,
        target: &mut DiagnosticCtx,
        collector_data_ctx: CollectorDiagDataCtx,
        resolver_data_ctx: ResolverDiagDataCtx,
    ) {
        self.collector.dctx.emit(target, collector_data_ctx);
        self.dctx.emit(target, resolver_data_ctx);
    }

    pub fn resolve(&mut self, tree: &HirItem) {
        // self.collector.level = CollectionLevel::Local;

        // let Some(scope_id) = self.collector.scope_ctx.get_id_by_hir(tree.id) else {
        //     bug!("expected a scope attached to the tree")
        // };

        let HirItemKind::Petal(tree) = &tree.kind else {
            bug!("invalid resolution! resolution must start at the tree (a petal)")
        };

        for item in &tree.items {
            if let Err(Some(diag)) = self.resolve_item(&item) {
                self.dctx.add(diag);
            }
        }
    }
}
