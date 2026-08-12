use std::{collections::HashMap, sync::Arc};

use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirId,
    def::{
        AdtDef, AdtKind, Binding, BuiltinKind, DefAccessibility, DefKind, Definition, FnDef,
        GenericParamDef, StructDef, StructFieldDef, VarDef,
    },
    item::{HirItem, HirItemKind, HirPetalKind, HirProtoItem, HirProtoItemAssocFnKind},
    scope::Scope,
};
use hycc_span::Span;
use hycc_ty::ty::{GenericArg, Ty};
use hycc_util::ternary;

use crate::collector::{CollectResult, Collector};

impl<'i> Collector<'i> {
    pub fn push_petal_item(&mut self, petal_item: &HirItem) -> CollectResult<usize> {
        let HirItemKind::Petal(petal) = &petal_item.kind else {
            unreachable!();
        };

        let mut pushed: usize = 0;
        match &petal.kind {
            HirPetalKind::File(path) | HirPetalKind::Inline(path) => {
                for segment in &path.segments {
                    let (def_id, defined) =
                        if let Some(def_id) = self.definitions.get_def_id(segment.id) {
                            Ok((def_id, true))
                        } else {
                            let def = Definition::new(
                                segment.ident.ident,
                                DefKind::Petal,
                                Some(self.petal_ctx.top_id()),
                                segment.id,
                                petal_item.span,
                                petal_item.accessibility,
                            );

                            ternary!(
                                petal.is_inline(),
                                self.try_define(def)
                                    .map(|(binding, defined)| (binding.def_id, defined)),
                                self.define(def).map(|binding| (binding.def_id, false))
                            )
                        }?;

                    self.definitions.define_id_hir(segment.id, def_id);

                    let petal_id = self.petal_ctx.try_create_child_petal(def_id);
                    self.petal_ctx.push(petal_id);

                    let scope_id = self.scope_ctx.try_attach_to_def(def_id, Scope::new());
                    self.scope_ctx.push_id(scope_id);

                    pushed += 1;

                    if !defined {
                        self.init_builtin();
                    }
                }
            }
            _ => {
                self.init_builtin();
            }
        };

        Ok(pushed)
    }

    pub fn pop_petals(&mut self, pushed: usize) {
        for _ in 0..pushed {
            self.scope_ctx.pop();
            self.petal_ctx.pop();
        }
    }

    pub fn enter_petal_scope<T>(
        &mut self,
        petal_item: &HirItem,
        mut f: impl FnMut(&mut Self) -> T,
    ) -> CollectResult<T> {
        let pushed = self.push_petal_item(&petal_item)?;
        let result = f(self);
        self.pop_petals(pushed);

        Ok(result)
    }

    pub fn collect_item(&mut self, item: &HirItem) -> CollectResult {
        match &item.kind {
            HirItemKind::Refer(_) => Ok(()),
            HirItemKind::Petal(_) => self.collect_petal(&item),
            HirItemKind::Proto(_) => {
                // self.collect_proto(&item)
                todo!()
            }
            HirItemKind::Extend(_) => self.collect_extend(&item),
            HirItemKind::Struct(_) => self.collect_struct(&item),
            HirItemKind::Fn(_) => self.collect_fn(&item),
            HirItemKind::VarDecl(_) => self.collect_var(&item),
        }
    }

    pub fn collect_petal(&mut self, petal_item: &HirItem) -> CollectResult {
        self.enter_petal_scope(&petal_item, |s| {
            let HirItemKind::Petal(petal) = &petal_item.kind else {
                unreachable!()
            };

            for item in &petal.items {
                if let Err(Some(diag)) = s.collect_item(&item) {
                    s.dctx.add(diag);
                }
            }
        })
    }

    pub fn collect_proto(&mut self, proto_item: &HirItem) -> CollectResult {
        if self.definitions.get_def_id(proto_item.id).is_some() {
            return Ok(());
        }

        let HirItemKind::Proto(proto) = &proto_item.kind else {
            unreachable!()
        };

        let def_id = self
            .define(Definition::new(
                proto.ident.ident,
                DefKind::Proto,
                Some(self.petal_ctx.top_id()),
                proto_item.id,
                proto_item.span,
                proto_item.accessibility,
            ))?
            .def_id;

        let scope_id = self.scope_ctx.try_attach_to_def(def_id, Scope::new());
        self.scope_ctx.push_id(scope_id);

        for item in &proto.items {
            if let Err(Some(diag)) = self.collect_proto_item(&item) {
                self.dctx.add(diag);
            }
        }

        self.scope_ctx.pop();
        Ok(())
    }

    fn collect_proto_item(&mut self, item: &HirProtoItem) -> CollectResult {
        match &item {
            HirProtoItem::AssocConst(decl) => self.collect_var(&decl),

            HirProtoItem::AssocFn(kind) => match &kind {
                HirProtoItemAssocFnKind::Sig(sig) => todo!(),
                HirProtoItemAssocFnKind::Impl(func) => self.collect_fn(&func),
            },

            #[allow(unreachable_patterns)]
            _ => todo!("collect proto item"),
        }
    }

    pub fn collect_extend(&mut self, extend_item: &HirItem) -> CollectResult {
        let HirItemKind::Extend(extend) = &extend_item.kind else {
            unreachable!()
        };

        let scope_id = self.scope_ctx.attach(extend_item.id, Scope::new());
        self.enter_scope(scope_id, |s| {
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
                        Some(s.petal_ctx.top_id()),
                        generic_param.id,
                        generic_param.span,
                        DefAccessibility::Priv,
                    ));

                    match res {
                        Ok(&Binding {
                            def_id: gp_def_id, ..
                        }) => {
                            let ty = Ty::new(
                                s.tctx.make_param_ty(
                                    gp_def_id,
                                    s.scope_ctx.generic_depth as u32,
                                    i as u32,
                                ),
                                generic_param.span,
                            );
                            s.tctx.attach_to_hir(generic_param.id, ty);
                        }

                        Err(diag) => {
                            diag.map(|diag| s.dctx.add(diag));
                        }
                    };
                }
            }

            // Define `Self`
            let self_sym = s.interner.intern("Self");
            s.define(Definition::new(
                self_sym,
                DefKind::Builtin(BuiltinKind::SelfTy),
                Some(s.petal_ctx.top_id()),
                extend.target.id,
                Span::default(),
                DefAccessibility::Priv,
            ));

            let scope_id = s.scope_ctx.attach(extend.target.id, Scope::new());
            s.enter_scope(scope_id, |s| {
                for item in &extend.items {
                    if let Err(Some(diag)) = s.collect_item(&item) {
                        s.dctx.add(diag);
                    }
                }
            });

            s.scope_ctx.generic_depth -= extend.generic_params.is_some() as u32;
        });

        Ok(())
    }

    pub fn collect_struct(&mut self, struct_item: &HirItem) -> CollectResult {
        if self.definitions.get_def_id(struct_item.id).is_some() {
            return Ok(());
        }

        let HirItemKind::Struct(strct) = &struct_item.kind else {
            unreachable!()
        };

        let def_id = self
            .define(Definition::new(
                strct.ident.ident,
                DefKind::Adt(Box::new(AdtDef::new(AdtKind::Struct(StructDef::new())))),
                Some(self.petal_ctx.top_id()),
                struct_item.id,
                struct_item.span,
                struct_item.accessibility,
            ))?
            .def_id;

        let mut adt_generic_args = Vec::new();
        let scope_id = self.scope_ctx.try_attach_to_def(def_id, Scope::new());
        self.enter_scope(scope_id, |s| {
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
                        Some(s.petal_ctx.top_id()),
                        generic_param.id,
                        generic_param.span,
                        DefAccessibility::Priv,
                    ));

                    match res {
                        Ok(&Binding {
                            def_id: gp_def_id, ..
                        }) => {
                            let adt_def = &mut s.definitions.get_mut(def_id).kind.expect_mut_adt();
                            // TODO: determine the param to be created and attached based on the
                            // generic parameter kind
                            let ty = Ty::new(
                                s.tctx.make_param_ty(
                                    gp_def_id,
                                    s.scope_ctx.generic_depth as u32,
                                    i as u32,
                                ),
                                generic_param.span,
                            );

                            adt_generic_args.push(GenericArg::Ty(ty.id));
                            s.tctx.attach_to_hir(generic_param.id, ty);

                            adt_def.generic_params.push(gp_def_id);
                        }

                        Err(diag) => {
                            diag.map(|diag| s.dctx.add(diag));
                        }
                    };
                }
            }

            s.scope_ctx.generic_depth -= strct.generic_params.is_some() as u32;
        });

        let ty = Ty::new(
            self.tctx.make_adt_ty(def_id, adt_generic_args.into()),
            struct_item.span,
        );
        self.tctx.attach_to_hir(struct_item.id, ty.clone());
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

    pub fn collect_fn(&mut self, fn_item: &HirItem) -> CollectResult {
        let HirItemKind::Fn(func) = &fn_item.kind else {
            unreachable!()
        };

        let def_id = if let Some(def_id) = self.definitions.get_def_id(fn_item.id) {
            def_id
        } else {
            self.define(Definition::new(
                func.sig.ident.ident,
                DefKind::Fn(Box::new(FnDef::new(func.sig.ret_ty.map(|ty| ty.id)))),
                Some(self.petal_ctx.top_id()),
                fn_item.id,
                fn_item.span,
                fn_item.accessibility,
            ))?
            .def_id
        };

        let scope_id = self.scope_ctx.try_attach_to_def(def_id, Scope::new());
        self.enter_scope(scope_id, |s| {
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
                        Some(s.petal_ctx.top_id()),
                        generic_param.id,
                        generic_param.span,
                        DefAccessibility::Priv,
                    ));

                    match res {
                        Ok(&Binding {
                            def_id: gp_def_id, ..
                        }) => {
                            if let Some(fn_def) =
                                &mut s.definitions.get_mut(def_id).kind.get_mut_fn()
                            {
                                let ty = Ty::new(
                                    s.tctx.make_param_ty(
                                        gp_def_id,
                                        s.scope_ctx.generic_depth as u32,
                                        i as u32,
                                    ),
                                    generic_param.span,
                                );

                                s.tctx.attach_to_hir(generic_param.id, ty);
                                fn_def.generic_params.push(gp_def_id);
                            }
                        }

                        Err(diag) => {
                            diag.map(|diag| s.dctx.add(diag));
                        }
                    };
                }
            }

            // Define the function parameters
            for param in &func.sig.params.list {
                let res = s.define(Definition::new(
                    param.ident.ident,
                    DefKind::FnParam,
                    Some(s.petal_ctx.top_id()),
                    param.id,
                    param.span,
                    DefAccessibility::Priv,
                ));

                match res {
                    Ok(&Binding {
                        def_id: p_def_id, ..
                    }) => {
                        if let Some(fn_def) = &mut s.definitions.get_mut(def_id).kind.get_mut_fn() {
                            fn_def.params.push(p_def_id);
                        }
                    }

                    Err(diag) => {
                        diag.map(|diag| s.dctx.add(diag));
                    }
                };
            }

            s.scope_ctx.generic_depth -= func.sig.generic_params.is_some() as u32;
        });

        Ok(())
    }

    pub fn collect_var(&mut self, var_item: &HirItem) -> CollectResult {
        let HirItemKind::VarDecl(var) = &var_item.kind else {
            unreachable!()
        };

        // var.mutability,
        if self.definitions.get_def_id(var_item.id).is_none() {
            self.define(Definition::new(
                var.ident.ident,
                DefKind::Var(Box::new(VarDef::new(var_item.level, var.mutability))),
                Some(self.petal_ctx.top_id()),
                var_item.id,
                var_item.span,
                var_item.accessibility,
            ))?;
        }

        Ok(())
    }
}
