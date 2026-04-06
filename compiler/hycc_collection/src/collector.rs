use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    def::{DefId, Definition, DefinitionTable},
    program::HirProgram,
};
use hycc_scope::ScopeCtx;

use crate::diag::{CollectorDiag, CollectorDiagCtx, CollectorDiagErrorKind};

#[derive(Debug)]
pub struct Collector {
    pub(crate) definitions: DefinitionTable,
    pub(crate) scope_ctx: ScopeCtx,
    pub dctx: CollectorDiagCtx,
}

pub type CollectResult<T = (), E = Option<CollectorDiag>> = Result<T, E>;

impl Collector {
    pub fn new() -> Self {
        Self {
            definitions: DefinitionTable::new(),
            scope_ctx: ScopeCtx::new(),
            dctx: CollectorDiagCtx::new(),
        }
    }

    pub fn define(&mut self, definition: Definition) -> CollectResult<DefId> {
        let top = self.scope_ctx.top_mut();

        let (name, space) = (definition.name, definition.kind.space());
        if let Some(earlier_def) = top.get(space, name) {
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
