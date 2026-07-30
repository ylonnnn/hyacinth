use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    def::{
        AdtKind, Binding, DefAccessibility, DefKind, Definition, FnDef, StructDef, StructFieldDef,
    },
    item::{HirItem, HirItemKind, HirPetalKind, HirProtoItem, HirProtoItemAssocFnKind},
    scope::Scope,
};
use hycc_ty::ty::Ty;
use hycc_util::{bug, ternary};

use crate::{
    collector::{CollectResult, CollectionLevel, Collector},
    extension::Extension,
};

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
        if self.ext_table.get_hir_ext_id(extend_item.id).is_some() {
            return Ok(());
        }

        let HirItemKind::Extend(extend) = &extend_item.kind else {
            unreachable!()
        };

        // NOTE: Currently, extension items are not collected during collection as the
        // target of the extension cannot be resolved as of this point.
        // Extension items are pre-collected during the resolution of the
        // extension.

        self.scope_ctx.attach(extend_item.id, Scope::new());
        self.ext_table.attach(
            extend_item.id,
            Extension {
                target: extend.target.id,
                items: extend.items.iter().map(|item| item.id).collect::<Vec<_>>(),
            },
        );

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
                DefKind::Adt(AdtKind::Struct(Box::new(StructDef::new()))),
                Some(self.petal_ctx.top_id()),
                struct_item.id,
                struct_item.span,
                struct_item.accessibility,
            ))?
            .def_id;

        self.scope_ctx.try_attach_to_def(def_id, Scope::new());

        let ty = Ty::new(self.tctx.make_adt_ty(def_id), struct_item.span);
        self.tctx.attach_to_hir(struct_item.id, ty.clone());
        // self.tctx.attach_to_def(def_id, ty);

        let DefKind::Adt(AdtKind::Struct(def)) = &mut self.definitions.get_mut(def_id).kind else {
            unreachable!()
        };

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

        let (def_id, collected) = if let Some(def_id) = self.definitions.get_def_id(fn_item.id) {
            (def_id, true)
        } else {
            (
                self.define(Definition::new(
                    func.sig.ident.ident,
                    DefKind::Fn(Box::new(FnDef::new(func.sig.ret_ty.map(|ty| ty.id)))),
                    Some(self.petal_ctx.top_id()),
                    fn_item.id,
                    fn_item.span,
                    fn_item.accessibility,
                ))?
                .def_id,
                false,
            )
        };

        let scope_id = self.scope_ctx.try_attach_to_def(def_id, Scope::new());
        self.enter_scope(scope_id, |s| {
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
                        if let DefKind::Fn(def) = &mut s.definitions.get_mut(def_id).kind {
                            def.params.push(p_def_id);
                        }
                    }
                    Err(diag) => {
                        diag.map(|diag| s.dctx.add(diag));
                    }
                };
            }
        });

        Ok(())
    }

    pub fn collect_var(&mut self, var_item: &HirItem) -> CollectResult {
        let HirItemKind::VarDecl(var) = &var_item.kind else {
            unreachable!()
        };

        if self.definitions.get_def_id(var_item.id).is_none() {
            self.define(Definition::new(
                var.ident.ident,
                DefKind::Var,
                Some(self.petal_ctx.top_id()),
                var_item.id,
                var_item.span,
                var_item.accessibility,
            ))?;
        }

        Ok(())
    }
}
