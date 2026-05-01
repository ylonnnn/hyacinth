use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    def::{
        BuiltinIntTy, BuiltinKind, BuiltinTyKind, DefAccessibility, DefId, Definition,
        DefinitionTable,
    },
    item::HirPetal,
};
use hycc_scope::{ScopeCtx, ScopeId};
use hycc_span::Span;
use hycc_symbol::SymbolInterner;
use hycc_ty::{context::TyCtx, ty::Ty};
use hycc_util::ternary;

use crate::diag::{CollectorDiag, CollectorDiagCtx, CollectorDiagErrorKind};

#[derive(Debug)]
pub struct Collector {
    pub tctx: TyCtx,
    pub scope_ctx: ScopeCtx,
    pub definitions: DefinitionTable,
    pub dctx: CollectorDiagCtx,

    // Default to top-level collection
    pub level: CollectionLevel,
    // The current collection level of the current node
    pub node_level: CollectionLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionLevel {
    Top,
    Local,
}

pub type CollectResult<T = (), E = Option<CollectorDiag>> = Result<T, E>;

impl Collector {
    pub fn new() -> Self {
        Self {
            definitions: DefinitionTable::new(),
            scope_ctx: ScopeCtx::new(),
            dctx: CollectorDiagCtx::new(),
            tctx: TyCtx::new(),
            level: CollectionLevel::Top,
            node_level: CollectionLevel::Top,
        }
    }

    pub fn init_builtin_ty(&mut self, interner: &mut SymbolInterner) {
        // Integers
        for signed in [true, false] {
            let prefix = ternary!(signed, "i", "u");
            for width in [8, 16, 32, 64, u8::MAX] {
                let b_ty = BuiltinTyKind::Int(BuiltinIntTy::Fixed(width, signed));
                let ty = Ty::new(self.tctx.make_builtin_ty(&b_ty), Span::default());

                let def = Definition::builtin(
                    interner.intern(&format!(
                        "{}{}",
                        prefix,
                        ternary!(width == u8::MAX, "size".into(), width.to_string())
                    )),
                    BuiltinKind::Ty(b_ty),
                    DefAccessibility::Pub,
                );

                match self.define(def) {
                    Ok(def_id) => self.tctx.attach_to_def(def_id, ty),
                    Err(Some(diag)) => {
                        self.dctx.add(diag);
                    }
                    _ => {}
                }
            }
        }

        // Float
        for width in [8, 16, 32, 64] {
            let b_ty = BuiltinTyKind::Float(width);
            let ty = Ty::new(self.tctx.make_builtin_ty(&b_ty), Span::default());

            let def = Definition::builtin(
                interner.intern(&format!("f{}", width.to_string())),
                BuiltinKind::Ty(b_ty),
                DefAccessibility::Pub,
            );

            match self.define(def) {
                Ok(def_id) => self.tctx.attach_to_def(def_id, ty),
                Err(Some(diag)) => {
                    self.dctx.add(diag);
                }
                _ => {}
            }
        }

        for (name, b_ty) in [
            ("bool", BuiltinTyKind::Bool),
            ("char", BuiltinTyKind::Char),
            ("str", BuiltinTyKind::String),
        ] {
            let ty = Ty::new(self.tctx.make_builtin_ty(&b_ty), Span::default());
            let def = Definition::builtin(
                interner.intern(name),
                BuiltinKind::Ty(b_ty),
                DefAccessibility::Pub,
            );

            match self.define(def) {
                Ok(def_id) => self.tctx.attach_to_def(def_id, ty),
                Err(Some(diag)) => {
                    self.dctx.add(diag);
                }
                _ => {}
            }
        }

        if let Err(Some(diag)) = self.define(Definition::builtin(
            interner.intern("_"),
            BuiltinKind::Ty(BuiltinTyKind::Infer),
            DefAccessibility::Pub,
        )) {
            self.dctx.add(diag);
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

    pub fn enter_scope<F, U>(
        &mut self,
        scope_id: ScopeId,
        level: CollectionLevel,
        mut handler: F,
    ) -> U
    where
        F: FnMut(&mut Self) -> U,
    {
        let prev_node_level = self.node_level;

        self.node_level = level;
        self.scope_ctx.push_id(scope_id);

        let data = handler(self);

        self.scope_ctx.pop();
        self.node_level = prev_node_level;

        data
    }

    pub fn is_expected_to_be_collected(&self) -> bool {
        self.level == CollectionLevel::Local && self.node_level == CollectionLevel::Top
    }

    pub fn collect_top(&mut self, tree: &HirPetal) {
        // Top-level collection
        self.level = CollectionLevel::Top;
        self.collect_tree(tree);
    }

    pub fn collect_local(&mut self, tree: &HirPetal) {
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
