use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
    sync::Arc,
};

use hycc_diagnostic::diagnostic::{Diagnostics, FromResultEmitter};
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
    ctx::TyCtx,
    extension::Extension,
    ty::{GenericArg, InferKind, Ty},
};
use hycc_util::{bug, ternary};

use crate::diag::{ResolveResult, ResolverDiag, ResolverDiagCtx, ResolverDiagErrorKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectorLevel {
    Top,
    Local,
}

#[derive(Debug)]
pub struct Collector<'c> {
    pub scope_ctx: ScopeCtx,
    pub petal_ctx: &'c mut PetalCtx,
    pub tctx: &'c mut TyCtx,
    pub definitions: &'c mut DefinitionTable,
    pub interner: &'c mut SymbolInterner,
    pub(crate) level: CollectorLevel,
}

impl<'c> Collector<'c> {
    pub fn new(
        petal_ctx: &'c mut PetalCtx,
        interner: &'c mut SymbolInterner,
        tctx: &'c mut TyCtx,
        definitions: &'c mut DefinitionTable,
    ) -> Self {
        Self {
            scope_ctx: ScopeCtx::new(),
            petal_ctx,
            tctx,
            definitions,
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
        let pushed = self.scope_ctx.push_id(scope_id);

        let data = handler(self);

        self.level = prev_level;
        if pushed {
            self.scope_ctx.pop();
        }

        data
    }

    pub fn collect(&mut self, tree: &HirItem, dctx: &mut ResolverDiagCtx) {
        let petal = tree.expect_petal();

        self.collect_petal(&tree, dctx).emit(dctx);
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
            HirItemKind::VarDecl(decl) => ternary!(
                self.level == CollectorLevel::Top,
                self.collect_var_decl(&item, dctx),
                decl.val
                    .as_ref()
                    .map_or_else(|| Ok(()), |val| self.collect_expr(&val, dctx))
            ),
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

                    let Some(binding) = res.emit(dctx) else {
                        continue;
                    };

                    let ty_id = s.tctx.make_param_ty(
                        binding.def_id,
                        s.scope_ctx.generic_depth as u32,
                        i as u32,
                    );
                    s.tctx
                        .attach_to_hir(generic_param.id, Ty::new(ty_id, generic_param.span));
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
                extend
                    .items
                    .iter()
                    .for_each(|item| s.collect_item(&item, dctx).emit_discard(dctx))
            });

            s.tctx.ext_table.attach_hir_ext(
                item.id,
                Extension::new(
                    item.id,
                    None,
                    std::mem::take(s.scope_ctx.get_mut(scope_id))
                        .all()
                        .into_iter()
                        .collect(),
                ),
            );

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
            self.collect_item(&item, dctx).emit(dctx);
        }

        for _ in 0..petals.len() {
            self.scope_ctx.pop();
            self.petal_ctx.pop();
        }

        Ok(())
    }

    // pub fn collect_proto(&mut self, proto_item: &HirItem) -> CollectResult {
    //     if self.definitions.get_def_id(proto_item.id).is_some() {
    //         return Ok(());
    //     }

    //     let HirItemKind::Proto(proto) = &proto_item.kind else {
    //         unreachable!()
    //     };

    //     let def_id = self
    //         .define(Definition::new(
    //             proto.ident.ident,
    //             DefKind::Proto,
    //             Some(self.petal_ctx.top_id()),
    //             proto_item.id,
    //             proto_item.span,
    //             proto_item.accessibility,
    //         ))?
    //         .def_id;

    //     let scope_id = self.scope_ctx.try_attach_to_def(def_id, Scope::new());
    //     self.scope_ctx.push_id(scope_id);

    //     for item in &proto.items {
    //         }
    //     }

    //     self.scope_ctx.pop();
    //     Ok(())
    // }

    // fn collect_proto_item(&mut self, item: &HirProtoItem) -> CollectResult {
    //     match &item {
    //         HirProtoItem::AssocConst(decl) => self.collect_var(&decl),

    //         HirProtoItem::AssocFn(kind) => match &kind {
    //             HirProtoItemAssocFnKind::Sig(sig) => todo!(),
    //             HirProtoItemAssocFnKind::Impl(func) => self.collect_fn(&func),
    //         },

    //         #[allow(unreachable_patterns)]
    //         _ => todo!("collect proto item"),
    //     }
    // }

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

                    let Some(binding) = res.emit(dctx) else {
                        continue;
                    };

                    let adt_def = s.definitions.get_mut(def_id).kind.expect_mut_adt();
                    adt_def.generic_params.push(binding.def_id);

                    // TODO: determine the param to be created and attached based on the
                    // generic parameter kind
                    let ty_id = s.tctx.make_param_ty(
                        binding.def_id,
                        s.scope_ctx.generic_depth as u32,
                        i as u32,
                    );

                    adt_generic_args.push(GenericArg::Ty(ty_id));
                    s.tctx
                        .attach_to_hir(generic_param.id, Ty::new(ty_id, generic_param.span));
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
        self.enter_scope(scope_id, CollectorLevel::Local, |s| {
            // Define function generic parameters
            let generic_param_tys = func
                .sig
                .generic_params
                .as_ref()
                .map(|generic_params| {
                    s.scope_ctx.generic_depth += 1;

                    generic_params
                        .list
                        .iter()
                        .enumerate()
                        .filter_map(|(i, generic_param)| {
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

                            let Some(binding) = res.emit(dctx) else {
                                return None;
                            };

                            let fn_def = s.definitions.get_mut(def_id).kind.expect_mut_fn();
                            fn_def.generic_params.push(binding.def_id);

                            let ty_id = s.tctx.make_param_ty(
                                binding.def_id,
                                s.scope_ctx.generic_depth as u32,
                                i as u32,
                            );

                            s.tctx.attach_to_hir(
                                generic_param.id,
                                Ty::new(ty_id, generic_param.span),
                            );
                            Some(GenericArg::Ty(ty_id))
                        })
                        .collect::<Arc<_>>()
                })
                .unwrap_or_else(|| Vec::new().into());

            // Define the function parameters
            func.sig.params.list.iter().for_each(|param| {
                let res = s.define(Definition::new(
                    param.ident.ident,
                    DefKind::FnParam,
                    s.petal_ctx.top_id(),
                    param.id,
                    param.span,
                    DefAccessibility::Priv,
                ));

                let Some(binding) = res.emit(dctx) else {
                    return;
                };

                s.definitions
                    .get_mut(def_id)
                    .kind
                    .expect_mut_fn()
                    .params
                    .push(binding.def_id);
            });

            let fn_ty_id = s.tctx.make_inferred_ty(item.span, InferKind::Any);
            s.tctx.attach_to_hir(item.id, Ty::new(fn_ty_id, item.span));

            s.scope_ctx.attach_id(func.body.id, scope_id);
            s.collect_block(&func.body, dctx).emit(dctx);

            s.scope_ctx.generic_depth -= func.sig.generic_params.is_some() as u32;
        });

        Ok(())
    }

    pub(crate) fn collect_var_decl(
        &mut self,
        item: &HirItem,
        dctx: &mut ResolverDiagCtx,
    ) -> ResolveResult {
        let var = item.expect_var();

        if var.ident.ident != self.interner.intern("_") {
            self.define(Definition::new(
                var.ident.ident,
                DefKind::Var(Box::new(VarDef::new(item.level, var.mutability))),
                self.petal_ctx.top_id(),
                item.id,
                item.span,
                item.accessibility,
            ))
            .emit(dctx);
        }

        let infer_ty_id = self.tctx.make_inferred_ty(item.span, InferKind::Any);
        self.tctx.attach_to_hir(
            item.id,
            Ty::new(infer_ty_id, var.ty.map_or_else(|| item.span, |ty| ty.span)),
        );

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
                s.collect_stmt(&stmt, dctx).emit(dctx);
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
            HirStmtKind::Ret(ret) => ret
                .value
                .as_ref()
                .map_or_else(|| Ok(()), |val| self.collect_expr(&val, dctx)),
            HirStmtKind::Pass(pass) => pass
                .value
                .as_ref()
                .map_or_else(|| Ok(()), |val| self.collect_expr(&val, dctx)),

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
                self.collect_expr(&left, dctx).emit(dctx);

                self.collect_expr(&right, dctx)
            }

            HirExprKind::Unary(unary) => self.collect_expr(&unary.expr(), dctx),

            HirExprKind::Assign(assignee, expr) => {
                self.collect_expr(&assignee, dctx).emit(dctx);

                self.collect_expr(&expr, dctx)
            }

            HirExprKind::Block(block) => self.collect_block(&block, dctx),

            HirExprKind::Array(array) => {
                for element in &array.elements {
                    self.collect_expr(&element, dctx).emit(dctx);
                }

                Ok(())
            }

            HirExprKind::Tuple(tup) => {
                for element in &tup.elements {
                    self.collect_expr(&element, dctx).emit(dctx);
                }

                Ok(())
            }

            HirExprKind::Struct(strct) => {
                for field in &strct.fields {
                    self.collect_expr(&field.val, dctx).emit(dctx);
                }

                Ok(())
            }

            HirExprKind::AnonFn(anfn) => {
                let scope_id = self.scope_ctx.attach(expr.id, Scope::new());
                self.enter_scope(scope_id, CollectorLevel::Local, |s| {
                    for param in &anfn.params.list {
                        s.define(Definition::new(
                            param.ident.ident,
                            DefKind::FnParam,
                            s.petal_ctx.top_id(),
                            param.id,
                            param.span,
                            DefAccessibility::Priv,
                        ))
                        .emit(dctx);

                        let Some(p_ty) = param.ty else {
                            continue;
                        };
                    }

                    s.scope_ctx.attach_id(anfn.body.id, scope_id);
                    s.collect_block(&anfn.body, dctx)
                })
            }

            HirExprKind::FnCall(call) => {
                let dctx = dctx;
                self.collect_expr(&call.callee, dctx).emit(dctx);

                for argument in &call.arguments.data {
                    self.collect_expr(&argument, dctx).emit(dctx);
                }

                Ok(())
            }

            HirExprKind::FieldAccess(access) => self.collect_expr(&access.leading, dctx),

            HirExprKind::MethodCall(call) => {
                self.collect_expr(&call.receiver, dctx).emit(dctx);

                for argument in &call.arguments.data {
                    self.collect_expr(&argument, dctx).emit(dctx);
                }

                Ok(())
            }

            HirExprKind::If(ite) => {
                self.collect_expr(&ite.cond, dctx).emit(dctx);

                self.collect_block(&ite.consequent, dctx).emit(dctx);

                ite.alternate.as_ref().map(|alternate| {
                    self.collect_block(&alternate, dctx).emit(dctx);
                });

                Ok(())
            }
        }
    }
}
