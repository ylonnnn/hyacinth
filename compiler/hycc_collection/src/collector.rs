use hycc_diagnostic::DiagnosticCtx;
use hycc_hir::{
    def::{DefId, Definition, DefinitionTable},
    program::HirProgram,
};
use hycc_scope::ScopeCtx;

use crate::error::{CollectionError, CollectionErrorKind};

#[derive(Debug)]
pub struct Collector<'d> {
    pub(crate) definitions: DefinitionTable,
    pub(crate) scope_ctx: ScopeCtx,
    pub(crate) dctx: &'d mut DiagnosticCtx,
}

pub type CollectResult<T = (), E = CollectionError> = Result<T, E>;

impl<'d> Collector<'d> {
    pub fn new(dctx: &'d mut DiagnosticCtx) -> Self {
        Self {
            definitions: DefinitionTable::new(),
            scope_ctx: ScopeCtx::new(),
            dctx,
        }
    }

    pub fn define(&mut self, definition: Definition) -> CollectResult<DefId> {
        let top = self.scope_ctx.top_mut();

        let (name, space) = (definition.name, definition.kind.space());
        if top.get(space, name).is_some() {
            Err(CollectionError::new(
                CollectionErrorKind::Duplication { ident: name },
                definition.span,
            ))
        } else {
            let def_id = self.definitions.define_hir(definition.hir_id, definition);
            top.define(space, name, def_id);

            Ok(def_id)
        }
    }

    pub fn collect(&mut self, tree: HirProgram) {
        for item in &tree.items {
            if let Err(err) = self.collect_item(item) {
                todo!("handle errors: {err:?}");
            }
        }
    }
}
