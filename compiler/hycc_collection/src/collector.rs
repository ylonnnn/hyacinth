use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirTable,
    def::{DefId, Definition, DefinitionTable},
    program::HirProgram,
};
use hycc_scope::ScopeCtx;

use crate::diag::{CollectorDiag, CollectorDiagCtx, CollectorDiagErrorKind};

#[derive(Debug)]
pub struct Collector<'t, 'h> {
    pub definitions: DefinitionTable,
    pub scope_ctx: ScopeCtx,
    pub dctx: CollectorDiagCtx,
    pub(crate) hir_table: &'t HirTable<'h>,
}

pub type CollectResult<T = (), E = Option<CollectorDiag>> = Result<T, E>;

impl<'t, 'h> Collector<'t, 'h> {
    pub fn new(hir_table: &'t HirTable<'h>) -> Self {
        Self {
            definitions: DefinitionTable::new(),
            scope_ctx: ScopeCtx::new(),
            dctx: CollectorDiagCtx::new(),
            hir_table,
        }
    }

    pub fn define(&mut self, definition: Definition) -> CollectResult<DefId> {
        let top = self.scope_ctx.top_mut();

        let (name, space) = (definition.name, definition.kind.space());
        if let Some(earlier_def) = top.get(space, name) {
            // let def = self.definitions.get(earlier_def);

            Err(Some(CollectorDiag::error(
                definition.span,
                CollectorDiagErrorKind::Duplication {
                    ident: name,
                    earlier_def,
                },
            )))
        } else {
            let def_id = self.definitions.define_hir(definition.hir_id, definition);
            top.define(space, name, def_id);

            Ok(def_id)
        }
    }

    pub fn collect(&mut self, tree: &HirProgram) {
        for item in &tree.items {
            if let Err(Some(err)) = self.collect_item(item) {
                self.dctx.add(err);
            }
        }
    }
}
