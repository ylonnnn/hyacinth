use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirId,
    block::HirBlock,
    def::{
        AdtDef, AdtKind, Binding, BuiltinIntTy, BuiltinKind, BuiltinTyKind, DefAccessibility,
        DefId, DefKind, DefPubAccessibilityKind, DefSpace, Definition, DefinitionTable, FnDef,
        GenericParamDef, StructDef, StructFieldDef, VarDef,
    },
    expr::{HirExpr, HirExprKind},
    item::{HirItem, HirItemKind, HirPetal, HirPetalKind},
    petal::PetalCtx,
    scope::{Scope, ScopeCtx, ScopeId},
    stmt::{HirStmt, HirStmtKind},
};
use hycc_span::Span;
use hycc_symbol::{Symbol, SymbolInterner};
use hycc_ty::{
    context::TyCtx,
    ty::{GenericArg, Ty},
};
use hycc_util::{bug, ternary};

use crate::diag::{ResolveResult, ResolverDiag, ResolverDiagCtx, ResolverDiagErrorKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectorLevel {
    Top,
    Local,
}

pub struct Collector<'i> {
    pub tctx: TyCtx,
    pub scope_ctx: ScopeCtx,
    pub definitions: DefinitionTable,
    pub petal_ctx: PetalCtx,
    pub interner: &'i mut SymbolInterner,
    pub(crate) level: CollectorLevel,
}

impl<'i> Collector<'i> {
    pub fn new(interner: &'i mut SymbolInterner) -> Self {
        Self {
            tctx: TyCtx::new(),
            scope_ctx: ScopeCtx::new(),
            definitions: DefinitionTable::new(),
            petal_ctx: PetalCtx::new(),
            interner,
            level: CollectorLevel::Top,
        }
    }

    pub fn init_builtin(&mut self, dctx: &mut ResolverDiagCtx) {
        if self.definitions.builtins.is_empty() {
            // Integers
            for signed in [true, false] {
                let prefix = ternary!(signed, "i", "u");
                for width in [8, 16, 32, 64, u8::MAX] {
                    let size = width == u8::MAX;

                    let b_ty = BuiltinTyKind::Int(ternary!(
                        size,
                        BuiltinIntTy::Size(signed),
                        BuiltinIntTy::Fixed(width, signed)
                    ));
                    let ty = Ty::new(self.tctx.make_builtin_ty(&b_ty), Span::default());

                    let def = Definition::builtin(
                        self.interner.intern(&format!(
                            "{}{}",
                            prefix,
                            ternary!(size, String::from("size"), width.to_string())
                        )),
                        BuiltinKind::Ty(b_ty),
                        self.petal_ctx.top_id(),
                        DefAccessibility::Priv,
                    );

                    match self.define(def) {
                        Ok(Binding { def_id, .. }) => {
                            self.scope_ctx.attach_to_def(def_id, Scope::new());
                            self.tctx.attach_to_def(def_id, ty);
                            self.definitions.builtins.push(def_id);
                        }
                        Err(diag) => {
                            dctx.add(diag);
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
                    self.petal_ctx.top_id(),
                    DefAccessibility::Priv,
                );

                match self.define(def) {
                    Ok(Binding { def_id, .. }) => {
                        self.scope_ctx.attach_to_def(def_id, Scope::new());
                        self.tctx.attach_to_def(def_id, ty);
                        self.definitions.builtins.push(def_id);
                    }
                    Err(diag) => {
                        dctx.add(diag);
                    }
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
                    self.petal_ctx.top_id(),
                    DefAccessibility::Priv,
                );

                match self.define(def) {
                    Ok(Binding { def_id, .. }) => {
                        self.scope_ctx.attach_to_def(def_id, Scope::new());
                        self.tctx.attach_to_def(def_id, ty);
                        self.definitions.builtins.push(def_id);
                    }
                    Err(diag) => {
                        dctx.add(diag);
                    }
                }
            }

            {
                // Infer
                let infer_sym = self.interner.intern("_");
                let def = Definition::builtin(
                    infer_sym,
                    BuiltinKind::Ty(BuiltinTyKind::Infer),
                    self.petal_ctx.top_id(),
                    DefAccessibility::Priv,
                );

                match self.define(def) {
                    Ok(Binding { def_id, .. }) => self.definitions.builtins.push(def_id),
                    Err(diag) => {
                        dctx.add(diag);
                    }
                }
            }
        } else {
            let builtin_defs = self
                .definitions
                .builtins
                .iter()
                .cloned()
                .collect::<Vec<_>>();

            for def_id in builtin_defs {
                let def = self.definitions.get(def_id);
                let (space, name) = (def.kind.space(), def.name);

                self.bind(space, name, Binding::new(def_id, DefAccessibility::Priv));
            }
        }

        // Petal-based Definitions
        {
            // spathe
            let root_id = self.petal_ctx.root_petal_id();
            let root_scope_id = self.petal_ctx.expect(root_id).scope_id(&self.scope_ctx);

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
                let super_scope_id = self.petal_ctx.expect(super_id).scope_id(&self.scope_ctx);
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

    // TODO: improve?
    pub fn bind(&mut self, space: DefSpace, name: Symbol, binding: Binding) -> bool {
        let scope_id = self.scope_ctx.resolve_redirection(self.scope_ctx.top_id());
        let scope = self.scope_ctx.get_mut(scope_id);

        if scope.get(Some(space), name).is_some() {
            let earlier_def = scope.get(Some(space), name).unwrap();
            self.definitions.get(earlier_def.def_id).hir_id
                == self.definitions.get(binding.def_id).hir_id
        } else {
            scope.define(space, name, binding);
            true
        }
    }

    pub fn try_define(&mut self, def: Definition) -> (Binding, bool) {
        let scope_id = self.scope_ctx.resolve_redirection(self.scope_ctx.top_id());
        let scope = self.scope_ctx.get_mut(scope_id);

        let (name, space) = (def.name, def.kind.space());
        if scope.get(Some(space), name).is_some() {
            (scope.get(Some(space), name).unwrap().clone(), false)
        } else {
            let accessibility = def.accessibility;
            let binding = Binding::new(self.definitions.define_hir(def.hir_id, def), accessibility);

            (scope.define(space, name, binding).clone(), true)
        }
    }

    pub fn define(&mut self, def: Definition) -> ResolveResult<Binding> {
        let (name, hir_id, span) = (def.name, def.hir_id, def.span);
        let (binding, defined) = self.try_define(def);

        ternary!(defined, Ok(binding), {
            if hir_id == self.definitions.get(binding.def_id).hir_id {
                Ok(binding)
            } else {
                Err(ResolverDiag::error(
                    span,
                    ResolverDiagErrorKind::Duplication {
                        ident: name,
                        earlier_def: binding.def_id,
                    },
                ))
            }
        })
    }

    pub fn enter_scope<F, U>(
        &mut self,
        scope_id: ScopeId,
        level: CollectorLevel,
        mut handler: F,
    ) -> U
    where
        F: FnMut(&mut Self) -> U,
    {
        let prev_level = (self.level, self.level = level).0;
        self.scope_ctx.push_id(scope_id);

        let data = handler(self);

        self.scope_ctx.pop();
        self.level = prev_level;

        data
    }

    pub fn collect(&mut self, tree: &HirItem, dctx: &mut ResolverDiagCtx) {
        let petal = tree.expect_petal();

        if let Err(diag) = self.collect_petal(&tree, dctx) {
            dctx.add(diag);
        }
    }

    pub(crate) fn collect_item(
        &mut self,
        item: &HirItem,
        dctx: &mut ResolverDiagCtx,
    ) -> ResolveResult {
        match &item.kind {
            HirItemKind::Refer(_) => Ok(()),
            HirItemKind::Petal(_) => self.collect_petal(&item, dctx),
            HirItemKind::Proto(_) => {
                // self.collect_proto(&item)
                todo!()
            }
            HirItemKind::Extend(_) => self.collect_extend(&item, dctx),
            HirItemKind::Struct(_) => self.collect_struct(&item, dctx),
            HirItemKind::Fn(_) => self.collect_fn(&item, dctx),
            HirItemKind::VarDecl(_) => ternary!(
                self.level == CollectorLevel::Top,
                self.collect_var(&item, dctx),
                Ok(())
            ),
            _ => todo!(),
        }
    }

    pub(crate) fn collect_extend(
        &mut self,
        item: &HirItem,
        dctx: &mut ResolverDiagCtx,
    ) -> ResolveResult {
        let extend = item.expect_extend();
        let scope_id = self.scope_ctx.attach(item.id, Scope::new());

        self.enter_scope(scope_id, CollectorLevel::Top, |s| {
            if let Some(generic_params) = &extend.generic_params {
                s.scope_ctx.generic_depth += 1;

                for (i, generic_param) in generic_params.list.iter().enumerate() {
                    let res = s.define(Definition::new(
                        generic_param.ident.ident,
                        DefKind::GenericParam(Box::new(GenericParamDef::new(
                            s.scope_ctx.generic_depth,
                            i as u32,
                            generic_param.kind,
                        ))),
                        s.petal_ctx.top_id(),
                        generic_param.id,
                        generic_param.span,
                        DefAccessibility::Priv,
                    ));

                    match res {
                        Ok(b) => {
                            let ty_id = s.tctx.make_param_ty(
                                b.def_id,
                                s.scope_ctx.generic_depth as u32,
                                i as u32,
                            );
                            s.tctx.attach_to_hir(
                                generic_param.id,
                                Ty::new(ty_id, generic_param.span),
                            );
                        }

                        Err(diag) => {
                            dctx.add(diag);
                        }
                    };
                }
            }

            // Define `Self`
            let self_sym = s.interner.intern("Self");
            s.define(Definition::new(
                self_sym,
                DefKind::Builtin(BuiltinKind::SelfTy),
                s.petal_ctx.top_id(),
                extend.target.id,
                Span::default(),
                DefAccessibility::Priv,
            ));

            let scope_id = s.scope_ctx.attach(extend.target.id, Scope::new());
            s.enter_scope(scope_id, CollectorLevel::Top, |s| {
                for item in &extend.items {
                    if let Err(diag) = s.collect_item(&item, dctx) {
                        dctx.add(diag);
                    }
                }
            });

            s.scope_ctx.generic_depth -= extend.generic_params.is_some() as u32;
        });

        Ok(())
    }

    pub(crate) fn collect_petal(
        &mut self,
        item: &HirItem,
        dctx: &mut ResolverDiagCtx,
    ) -> ResolveResult {
        let petal = item.expect_petal();

        let petals = match &petal.kind {
            HirPetalKind::File(path) | HirPetalKind::Inline(path) => {
                // TODO: throw errors if a segment contains generic arguments
                path.segments
                    .iter()
                    .map(|segment| {
                        self.definitions
                            .get_def_id(segment.id)
                            .map_or_else(
                                || -> ResolveResult<DefId, ResolverDiag> {
                                    let def = Definition::new(
                                        segment.ident.ident,
                                        DefKind::Petal,
                                        self.petal_ctx.top_id(),
                                        segment.id,
                                        segment.span,
                                        item.accessibility,
                                    );

                                    ternary!(
                                        !petal.is_inline(),
                                        self.define(def).map(|binding| binding.def_id),
                                        Ok(self.try_define(def).0.def_id)
                                    )
                                },
                                |def_id| Ok(def_id),
                            )
                            .map(|def_id| {
                                self.definitions.define_id_hir(segment.id, def_id);
                                self.scope_ctx.try_attach_to_def(def_id, Scope::new());
                                self.petal_ctx.try_create_child_petal(def_id)
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?
            }
            _ => {
                let root_scope_id = self.scope_ctx.root_id();
                self.scope_ctx.attach_id(item.id, root_scope_id);

                vec![self.petal_ctx.create_root_petal(root_scope_id)]
            }
        };

        for petal_id in &petals {
            self.petal_ctx.push(*petal_id);
            self.scope_ctx
                .push_id(self.petal_ctx.expect(*petal_id).scope_id(&self.scope_ctx));
        }

        // Define built-ins
        self.init_builtin(dctx);

        for item in &petal.items {
            if let Err(diag) = self.collect_item(&item, dctx) {
                dctx.add(diag);
            }
        }

        for _ in 0..petals.len() {
            self.scope_ctx.pop();
            self.petal_ctx.pop();
        }

        Ok(())
    }

    pub(crate) fn collect_struct(
        &mut self,
        item: &HirItem,
        dctx: &mut ResolverDiagCtx,
    ) -> ResolveResult {
        if self.definitions.get_def_id(item.id).is_some() {
            return Ok(());
        }

        let strct = item.expect_struct();
        let def_id = self
            .define(Definition::new(
                strct.ident.ident,
                DefKind::Adt(Box::new(AdtDef::new(AdtKind::Struct(StructDef::new())))),
                self.petal_ctx.top_id(),
                item.id,
                item.span,
                item.accessibility,
            ))?
            .def_id;

        let mut adt_generic_args = Vec::new();
        let scope_id = self.scope_ctx.try_attach_to_def(def_id, Scope::new());
        self.enter_scope(scope_id, CollectorLevel::Top, |s| {
            if let Some(generic_params) = &strct.generic_params {
                s.scope_ctx.generic_depth += 1;

                for (i, generic_param) in generic_params.list.iter().enumerate() {
                    let res = s.define(Definition::new(
                        generic_param.ident.ident,
                        DefKind::GenericParam(Box::new(GenericParamDef::new(
                            s.scope_ctx.generic_depth,
                            i as u32,
                            generic_param.kind,
                        ))),
                        s.petal_ctx.top_id(),
                        generic_param.id,
                        generic_param.span,
                        DefAccessibility::Priv,
                    ));

                    match res {
                        Ok(b) => {
                            let adt_def = s.definitions.get_mut(def_id).kind.expect_mut_adt();
                            adt_def.generic_params.push(b.def_id);

                            // TODO: determine the param to be created and attached based on the
                            // generic parameter kind
                            let ty_id = s.tctx.make_param_ty(
                                b.def_id,
                                s.scope_ctx.generic_depth as u32,
                                i as u32,
                            );

                            adt_generic_args.push(GenericArg::Ty(ty_id));
                            s.tctx.attach_to_hir(
                                generic_param.id,
                                Ty::new(ty_id, generic_param.span),
                            );
                        }

                        Err(diag) => {
                            dctx.add(diag);
                        }
                    };
                }
            }

            s.scope_ctx.generic_depth -= strct.generic_params.is_some() as u32;
        });

        let ty = Ty::new(
            self.tctx.make_adt_ty(def_id, adt_generic_args.into()),
            item.span,
        );
        self.tctx.attach_to_hir(item.id, ty.clone());
        // self.tctx.attach_to_def(def_id, ty);

        let def = &mut self
            .definitions
            .get_mut(def_id)
            .kind
            .expect_mut_adt()
            .expect_mut_struct();

        for field in &strct.fields.list {
            let name = field.ident.ident;
            if let Some(idx) = def.field_map.get(&name) {
                todo!("throw error: duplication: {idx:?}")
            };

            def.field_map.insert(name, def.fields.len());
            def.fields.push(StructFieldDef {
                name,
                accessibility: field.accessibility,
                span: field.span,
                ty: field.ty.id,
            });
        }

        Ok(())
    }

    pub(crate) fn collect_fn(
        &mut self,
        item: &HirItem,
        dctx: &mut ResolverDiagCtx,
    ) -> ResolveResult {
        let func = item.expect_fn();
        let def_id = self
            .define(Definition::new(
                func.sig.ident.ident,
                DefKind::Fn(Box::new(FnDef::new(func.sig.ret_ty.map(|ty| ty.id)))),
                self.petal_ctx.top_id(),
                item.id,
                item.span,
                item.accessibility,
            ))
            .map(|binding| binding.def_id)?;

        let scope_id = self.scope_ctx.try_attach_to_def(def_id, Scope::new());
        self.enter_scope(scope_id, CollectorLevel::Top, |s| {
            // Define function type parameters
            if let Some(generic_params) = &func.sig.generic_params {
                s.scope_ctx.generic_depth += 1;

                for (i, generic_param) in generic_params.list.iter().enumerate() {
                    let res = s.define(Definition::new(
                        generic_param.ident.ident,
                        DefKind::GenericParam(Box::new(GenericParamDef::new(
                            s.scope_ctx.generic_depth as u32,
                            i as u32,
                            generic_param.kind,
                        ))),
                        s.petal_ctx.top_id(),
                        generic_param.id,
                        generic_param.span,
                        DefAccessibility::Priv,
                    ));

                    match res {
                        Ok(b) => {
                            let fn_def = s.definitions.get_mut(def_id).kind.expect_mut_fn();
                            fn_def.generic_params.push(b.def_id);

                            let ty_id = s.tctx.make_param_ty(
                                b.def_id,
                                s.scope_ctx.generic_depth as u32,
                                i as u32,
                            );

                            s.tctx.attach_to_hir(
                                generic_param.id,
                                Ty::new(ty_id, generic_param.span),
                            );
                        }

                        Err(diag) => {
                            dctx.add(diag);
                        }
                    };
                }
            }

            // Define the function parameters
            for param in &func.sig.params.list {
                let res = s.define(Definition::new(
                    param.ident.ident,
                    DefKind::FnParam,
                    s.petal_ctx.top_id(),
                    param.id,
                    param.span,
                    DefAccessibility::Priv,
                ));

                match res {
                    Ok(b) => {
                        let fn_def = s.definitions.get_mut(def_id).kind.expect_mut_fn();
                        fn_def.params.push(b.def_id);
                    }

                    Err(diag) => {
                        dctx.add(diag);
                    }
                };
            }

            s.scope_ctx.attach_id(func.body.id, scope_id);
            if let Err(diag) = s.collect_block(&func.body, dctx) {
                dctx.add(diag);
            }

            s.scope_ctx.generic_depth -= func.sig.generic_params.is_some() as u32;
        });

        Ok(())
    }

    pub(crate) fn collect_var(
        &mut self,
        item: &HirItem,
        dctx: &mut ResolverDiagCtx,
    ) -> ResolveResult {
        let HirItemKind::VarDecl(var) = &item.kind else {
            unreachable!()
        };

        if self.definitions.get_def_id(item.id).is_none()
            && var.ident.ident != self.interner.intern("_")
        {
            self.define(Definition::new(
                var.ident.ident,
                DefKind::Var(Box::new(VarDef::new(item.level, var.mutability))),
                self.petal_ctx.top_id(),
                item.id,
                item.span,
                item.accessibility,
            ))?;
        }

        Ok(())
    }

    pub(crate) fn collect_block(
        &mut self,
        block: &HirBlock,
        dctx: &mut ResolverDiagCtx,
    ) -> ResolveResult {
        // Attempt to attach a scope to the HIR node. There are instances where
        // the block already has a scope attached such as function bodies where
        // the body and the function itself share the same scope.
        let scope_id = self.scope_ctx.try_attach(block.id, Scope::new());

        self.enter_scope(scope_id, CollectorLevel::Local, |s| {
            for stmt in &block.stmts {
                if let Err(diag) = s.collect_stmt(&stmt, dctx) {
                    dctx.add(diag);
                }
            }
        });

        Ok(())
    }

    pub(crate) fn collect_stmt(
        &mut self,
        stmt: &HirStmt,
        dctx: &mut ResolverDiagCtx,
    ) -> ResolveResult {
        match &stmt.kind {
            HirStmtKind::Ret(ret) => todo!("ret stmt"),
            HirStmtKind::Pass(pass) => todo!("pass stmt"),

            HirStmtKind::Expr(expr) => self.collect_expr(&expr, dctx),
            HirStmtKind::Item(item) => self.collect_item(&item, dctx),
        }
    }

    pub(crate) fn collect_expr(
        &mut self,
        expr: &HirExpr,
        dctx: &mut ResolverDiagCtx,
    ) -> ResolveResult {
        match &expr.kind {
            HirExprKind::Path(_) | HirExprKind::Literal(_) => Ok(()),
            HirExprKind::RefExpr(ref_expr) => self.collect_expr(&ref_expr.expr, dctx),

            HirExprKind::Binary(_, left, right) => {
                if let Err(diag) = self.collect_expr(&left, dctx) {
                    dctx.add(diag);
                }

                self.collect_expr(&right, dctx)
            }

            HirExprKind::Unary(unary) => self.collect_expr(&unary.expr(), dctx),

            HirExprKind::Assign(assignee, expr) => {
                if let Err(diag) = self.collect_expr(&assignee, dctx) {
                    dctx.add(diag);
                }

                self.collect_expr(&expr, dctx)
            }

            HirExprKind::Block(block) => self.collect_block(&block, dctx),

            HirExprKind::Array(array) => {
                for element in &array.elements {
                    if let Err(diag) = self.collect_expr(&element, dctx) {
                        dctx.add(diag);
                    }
                }

                Ok(())
            }

            HirExprKind::Tuple(tup) => {
                for element in &tup.elements {
                    if let Err(diag) = self.collect_expr(&element, dctx) {
                        dctx.add(diag);
                    }
                }

                Ok(())
            }

            HirExprKind::Struct(strct) => {
                for field in &strct.fields {
                    if let Err(diag) = self.collect_expr(&field.val, dctx) {
                        dctx.add(diag);
                    }
                }

                Ok(())
            }

            HirExprKind::AnonFn(anfn) => {
                // TODO: collect params
                self.collect_block(&anfn.body, dctx)
            }

            HirExprKind::FnCall(call) => {
                let dctx = dctx;
                if let Err(diag) = self.collect_expr(&call.callee, dctx) {
                    dctx.add(diag);
                }

                for argument in &call.arguments.data {
                    if let Err(diag) = self.collect_expr(&argument, dctx) {
                        dctx.add(diag);
                    }
                }

                Ok(())
            }

            HirExprKind::FieldAccess(access) => self.collect_expr(&access.leading, dctx),

            HirExprKind::MethodCall(call) => {
                if let Err(diag) = self.collect_expr(&call.receiver, dctx) {
                    dctx.add(diag);
                }

                for argument in &call.arguments.data {
                    if let Err(diag) = self.collect_expr(&argument, dctx) {
                        dctx.add(diag);
                    }
                }

                Ok(())
            }

            HirExprKind::If(ite) => {
                if let Err(diag) = self.collect_expr(&ite.cond, dctx) {
                    dctx.add(diag);
                }

                if let Err(diag) = self.collect_block(&ite.consequent, dctx) {
                    dctx.add(diag);
                }

                ite.alternate.as_ref().map(|alternate| {
                    if let Err(diag) = self.collect_block(&alternate, dctx) {
                        dctx.add(diag);
                    }
                });

                Ok(())
            }
        }
    }
}
