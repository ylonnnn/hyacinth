use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirTable,
    def::{
        BuiltinIntTy, BuiltinKind, BuiltinTyKind, DefAccessibility, DefId, Definition,
        DefinitionTable,
    },
    item::HirPetal,
};
use hycc_scope::{ScopeCtx, ScopeId};
use hycc_symbol::SymbolInterner;
use hycc_util::ternary;

use crate::diag::{CollectorDiag, CollectorDiagCtx, CollectorDiagErrorKind};

#[derive(Debug)]
pub struct Collector<'t, 'h> {
    pub definitions: DefinitionTable,
    pub scope_ctx: ScopeCtx,
    pub dctx: CollectorDiagCtx,
    #[allow(unused)]
    pub(crate) hir_table: &'t HirTable<'h>,

    // Default to top-level collection
    pub(crate) level: CollectionLevel,
    // The current collection level of the current node
    pub(crate) node_level: CollectionLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionLevel {
    Top,
    Local,
}

pub type CollectResult<T = (), E = Option<CollectorDiag>> = Result<T, E>;

impl<'t, 'h> Collector<'t, 'h> {
    pub fn new(hir_table: &'t HirTable<'h>) -> Self {
        Self {
            definitions: DefinitionTable::new(),
            scope_ctx: ScopeCtx::new(),
            dctx: CollectorDiagCtx::new(),
            hir_table,
            level: CollectionLevel::Top,
            node_level: CollectionLevel::Top,
        }
    }

    pub fn init_builtin_ty(&mut self, interner: &mut SymbolInterner) {
        // Integers
        for signed in [true, false] {
            let prefix = ternary!(signed, "i", "u");
            for width in [8, 16, 32, 64] {
                let def = Definition::builtin(
                    interner.intern(&format!("{}{}", prefix, width.to_string())),
                    BuiltinKind::Ty(BuiltinTyKind::Int(BuiltinIntTy::Fixed(width, signed))),
                    DefAccessibility::Pub,
                );

                if let Err(Some(diag)) = self.define(def) {
                    self.dctx.add(diag);
                }
            }

            // Pointer Size Integer
            let def = Definition::builtin(
                interner.intern(&format!("{}size", prefix)),
                BuiltinKind::Ty(BuiltinTyKind::Int(BuiltinIntTy::Size(signed))),
                DefAccessibility::Pub,
            );

            if let Err(Some(diag)) = self.define(def) {
                self.dctx.add(diag);
            }
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

    pub fn try_define(&mut self, definition: Definition) -> CollectResult<DefId> {
        let top = self.scope_ctx.top_mut();

        let (name, space) = (definition.name, definition.kind.space());
        if let Some(earlier_def) = top.get(space, name) {
            Ok(earlier_def)
        } else {
            let def_id = self.definitions.define_hir(definition.hir_id, definition);
            top.define(space, name, def_id);

            Ok(def_id)
        }
    }

    pub fn enter_scope<F>(&mut self, scope_id: ScopeId, level: CollectionLevel, mut handler: F)
    where
        F: FnMut(&mut Self),
    {
        let prev_node_level = self.node_level;

        self.node_level = level;
        self.scope_ctx.push_id(scope_id);

        handler(self);

        self.scope_ctx.pop();
        self.node_level = prev_node_level;
    }

    pub fn is_expected_to_be_collected(&self) -> bool {
        self.level == CollectionLevel::Local && self.node_level == CollectionLevel::Top
    }

    pub fn collect(&mut self, tree: &HirPetal) {
        // Top-level collection
        self.level = CollectionLevel::Top;
        self.collect_tree(tree);

        // Local definitions collection
        self.level = CollectionLevel::Local;
        self.collect_tree(tree);
    }

    fn collect_tree(&mut self, tree: &HirPetal) {
        self.node_level = CollectionLevel::Top;

        for item in &tree.items {
            if let Err(Some(diag)) = self.collect_item(item) {
                self.dctx.add(diag);
            }
        }
    }
}
