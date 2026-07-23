use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirId,
    def::{
        Binding, BuiltinIntTy, BuiltinKind, BuiltinTyKind, DefAccessibility, DefId, DefKind,
        DefPubAccessibilityKind, DefSpace, Definition, DefinitionTable,
    },
    item::{HirItem, HirItemKind},
    petal::{Petal, PetalCtx},
    scope::{Scope, ScopeCtx, ScopeId},
};
use hycc_span::Span;
use hycc_symbol::SymbolInterner;
use hycc_ty::{context::TyCtx, ty::Ty};
use hycc_util::{bug, ternary};

use crate::diag::{CollectorDiag, CollectorDiagCtx, CollectorDiagErrorKind};

#[derive(Debug)]
pub struct Collector<'i> {
    pub tctx: TyCtx,
    pub scope_ctx: ScopeCtx,
    pub definitions: DefinitionTable,
    pub dctx: CollectorDiagCtx,
    pub petal_ctx: PetalCtx,
    pub interner: &'i mut SymbolInterner,

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

impl<'i> Collector<'i> {
    pub fn new(interner: &'i mut SymbolInterner) -> Self {
        let mut inst = Self {
            definitions: DefinitionTable::new(),
            scope_ctx: ScopeCtx::new(),
            dctx: CollectorDiagCtx::new(),
            tctx: TyCtx::new(),
            petal_ctx: PetalCtx::new(),
            interner,
            level: CollectionLevel::Top,
            node_level: CollectionLevel::Top,
        };

        let root_petal_id = inst
            .petal_ctx
            .add_petal(Petal::Root(inst.scope_ctx.root_id()));
        inst.petal_ctx.push(root_petal_id);

        inst
    }

    pub fn init_builtin(&mut self) {
        // Integers
        for signed in [true, false] {
            let prefix = ternary!(signed, "i", "u");
            for width in [8, 16, 32, 64, u8::MAX] {
                let b_ty = BuiltinTyKind::Int(BuiltinIntTy::Fixed(width, signed));
                let ty = Ty::new(self.tctx.make_builtin_ty(&b_ty), Span::default());

                let def = Definition::builtin(
                    self.interner.intern(&format!(
                        "{}{}",
                        prefix,
                        ternary!(width == u8::MAX, "size".into(), width.to_string())
                    )),
                    BuiltinKind::Ty(b_ty),
                    Some(self.petal_ctx.top_id()),
                    DefAccessibility::Pub(DefPubAccessibilityKind::All),
                );

                match self.define(def) {
                    Ok(&Binding { def_id, .. }) => self.tctx.attach_to_def(def_id, ty),
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
                self.interner.intern(&format!("f{}", width.to_string())),
                BuiltinKind::Ty(b_ty),
                Some(self.petal_ctx.top_id()),
                DefAccessibility::Pub(DefPubAccessibilityKind::All),
            );

            match self.define(def) {
                Ok(&Binding { def_id, .. }) => self.tctx.attach_to_def(def_id, ty),
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
                self.interner.intern(name),
                BuiltinKind::Ty(b_ty),
                Some(self.petal_ctx.top_id()),
                DefAccessibility::Pub(DefPubAccessibilityKind::All),
            );

            match self.define(def) {
                Ok(&Binding { def_id, .. }) => self.tctx.attach_to_def(def_id, ty),
                Err(Some(diag)) => {
                    self.dctx.add(diag);
                }
                _ => {}
            }
        }

        let infer_sym = self.interner.intern("_");
        if let Err(Some(diag)) = self.define(Definition::builtin(
            infer_sym,
            BuiltinKind::Ty(BuiltinTyKind::Infer),
            Some(self.petal_ctx.top_id()),
            DefAccessibility::Pub(DefPubAccessibilityKind::All),
        )) {
            self.dctx.add(diag);
        }

        // Petal-based Definitions
        {
            // spathe
            let root_id = self.petal_ctx.root_petal_id();
            let root_scope_id = self.petal_ctx.get(root_id).scope_id(&self.scope_ctx);

            let spathe_def = Definition::new(
                self.interner.intern("spathe"),
                DefKind::Petal,
                None,
                HirId::Invalid,
                Span::default(),
                DefAccessibility::Pub(DefPubAccessibilityKind::This),
            );
            let spathe_def_id = self.define(spathe_def).unwrap().def_id;

            self.scope_ctx
                .attach_id_to_def(spathe_def_id, root_scope_id);
            self.petal_ctx.attach_petal_id(spathe_def_id, root_id);
        }

        {
            // super
            if let Some(super_id) = self.petal_ctx.from_top_id(1) {
                let super_scope_id = self.petal_ctx.get(super_id).scope_id(&self.scope_ctx);
                let super_def = Definition::new(
                    self.interner.intern("super"),
                    DefKind::Petal,
                    None,
                    HirId::Invalid,
                    Span::default(),
                    DefAccessibility::Pub(DefPubAccessibilityKind::This),
                );
                let super_def_id = self.define(super_def).unwrap().def_id;

                self.scope_ctx
                    .attach_id_to_def(super_def_id, super_scope_id);
                self.petal_ctx.attach_petal_id(super_def_id, super_id);
            };
        }
    }

    pub fn define(&mut self, definition: Definition) -> CollectResult<&Binding> {
        let top = self.scope_ctx.top_mut();

        let (name, space) = (definition.name, definition.kind.space());
        if let Some(earlier_def) = top.get(Some(space), name) {
            Err(Some(CollectorDiag::error(
                definition.span,
                CollectorDiagErrorKind::Duplication {
                    ident: name,
                    earlier_def: earlier_def.def_id,
                },
            )))
        } else {
            let accessibility = definition.accessibility;
            let binding = Binding::new(
                self.definitions.define_hir(definition.hir_id, definition),
                accessibility,
            );

            Ok(top.define(space, name, binding))
        }
    }

    pub fn try_define(&mut self, definition: Definition) -> CollectResult<(&Binding, bool)> {
        let top = self.scope_ctx.top_mut();
        let (name, space) = (definition.name, definition.kind.space());

        if top.get(Some(space), name).is_some() {
            Ok((top.get(Some(space), name).unwrap(), true))
        } else {
            let accessibility = definition.accessibility;
            let binding = Binding::new(
                self.definitions.define_hir(definition.hir_id, definition),
                accessibility,
            );

            Ok((top.define(space, name, binding), false))
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

    pub fn collect(&mut self, tree: &HirItem) {
        let hir_id = tree.id;
        let HirItemKind::Petal(_) = &tree.kind else {
            bug!("invalid item collection! collection must start at the tree (a petal)")
        };

        self.level = CollectionLevel::Top;
        self.node_level = CollectionLevel::Top;

        let root_scope_id = self.scope_ctx.root_id();
        self.scope_ctx.attach_id(hir_id, root_scope_id);

        if let Err(Some(diag)) = self.collect_petal(tree) {
            self.dctx.add(diag);
        }
    }
}
