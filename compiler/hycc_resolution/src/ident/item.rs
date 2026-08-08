use std::collections::HashMap;

use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirId, HirNode,
    def::{Binding, BuiltinKind, DefAccessibility, DefKind, DefResolution, DefSpace, Definition},
    item::{HirItem, HirItemKind, HirReferTarget, HirReferTargetKind},
    scope::{Scope, ScopeId},
    ty::HirTyKind,
};
use hycc_span::Span;
use hycc_ty::extension::{Extension, ExtensionTarget};
use hycc_util::bug;

use crate::{ResolveResult, ident::resolver::Resolver};

impl<'c, 'i, 'h> Resolver<'c, 'i, 'h> {
    pub fn enter_petal_scope<T>(
        &mut self,
        petal_item: &HirItem,
        mut f: impl FnMut(&mut Self) -> T,
    ) -> Option<T> {
        let pushed = self
            .collector
            .push_petal_item(&petal_item)
            .map_err(|err| {
                if let Some(diag) = err {
                    self.collector.dctx.add(diag);
                }

                Option::<T>::None
            })
            .ok()?;

        let result = f(self);
        self.collector.pop_petals(pushed);

        Some(result)
    }

    pub(crate) fn resolve_item(&mut self, item: &HirItem) -> ResolveResult {
        match &item.kind {
            HirItemKind::Refer(_) => self.resolve_refer(&item),
            HirItemKind::Petal(_) => self.resolve_petal(&item),
            HirItemKind::Proto(_) => todo!("(ident) resolve proto"),
            HirItemKind::Extend(_) => self.resolve_extend(&item),
            HirItemKind::Struct(_) => self.resolve_struct(&item),
            HirItemKind::Fn(_) => self.resolve_fn(&item),
            HirItemKind::VarDecl(_) => self.resolve_var_decl(&item),
        }
    }

    pub(crate) fn resolve_refer(&mut self, refer_item: &HirItem) -> ResolveResult {
        let HirItemKind::Refer(refer) = &refer_item.kind else {
            unreachable!();
        };

        if let Err(Some(diag)) =
            self.resolve_refer_target(&refer.target, refer_item.accessibility, None)
        {
            self.dctx.add(diag);
        }

        Ok(())
    }

    pub(crate) fn resolve_refer_target(
        &mut self,
        target: &HirReferTarget,
        accessibility: DefAccessibility,
        mut resolution: Option<DefResolution>,
    ) -> ResolveResult {
        match &target.kind {
            HirReferTargetKind::Child(alias) => {
                let target = target.symbol;
                let res = self.resolve_ident(&target, resolution)?.unwrap();

                // TODO: improve
                let DefResolution::Petal(def_id) = res else {
                    todo!("throw error: cannot `refer` to non-petal definitions")
                };

                let sym = target.ident.ident;
                let actual = self.collector.definitions.get(def_id);

                self.collector.scope_ctx.top_mut().define(
                    actual.kind.space(),
                    alias.unwrap_or(sym),
                    Binding::new(def_id, accessibility),
                );
            }

            HirReferTargetKind::Parent(children) => {
                resolution.replace(
                    self.expect_space(
                        DefSpace::Type,
                        |s| -> ResolveResult<Option<DefResolution>> {
                            s.resolve_ident(&target.symbol, resolution)
                        },
                    )?
                    .unwrap(),
                );

                for child in children {
                    if let Err(Some(diag)) =
                        self.resolve_refer_target(&child, accessibility, resolution)
                    {
                        self.dctx.add(diag);
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) fn resolve_petal(&mut self, petal_item: &HirItem) -> ResolveResult {
        self.enter_petal_scope(&petal_item, |s| {
            let HirItemKind::Petal(petal) = &petal_item.kind else {
                unreachable!()
            };

            for item in &petal.items {
                if let Err(Some(diag)) = s.resolve_item(&item) {
                    s.dctx.add(diag);
                }
            }
        });

        Ok(())
    }

    pub(crate) fn resolve_extend(&mut self, extend_item: &HirItem) -> ResolveResult {
        let HirItemKind::Extend(extend) = &extend_item.kind else {
            unreachable!()
        };

        if !extend_item.is_top_level() {
            if let Err(Some(diag)) = self.collector.collect_extend(&extend_item) {
                self.collector.dctx.add(diag);
            }
        }

        let ext = self.collector.ext_table.expect_hir_ext(extend_item.id);
        let scope_id = self.collector.scope_ctx.expect_hir_scope_id(extend_item.id);

        self.enter_scope(scope_id, |s| {
            if let Some(generic_params) = &extend.generic_params {
                for generic_param in &generic_params.list {
                    for proto_req in &generic_param.proto_reqs {
                        if let Err(Some(diag)) =
                            s.expect_space(DefSpace::Type, |s| s.resolve_path(proto_req))
                        {
                            s.dctx.add(diag);
                        }
                    }
                }
            }

            // Resolve extension target
            if let Err(Some(diag)) = s.resolve_ty(&extend.target) {
                s.dctx.add(diag);
            }

            // Resolve extension items
            for item in &extend.items {
                if let Err(Some(diag)) = s.resolve_item(&item) {
                    s.dctx.add(diag);
                }
            }
        });

        if let HirTyKind::Path(path) = &extend.target.kind {
            let def_id = self.collector.definitions.expect_def_id(path.id);
            let def_petal = self.collector.definitions.get(def_id).petal;

            let target = ExtensionTarget::Def(def_id);

            let ext_id = self.collector.tctx.ext_table.attach(
                target,
                Extension::new(
                    extend_item.id,
                    target,
                    None,
                    std::mem::take(
                        self.collector
                            .scope_ctx
                            .expect_hir_mut_scope(extend.target.id),
                    )
                    .all()
                    .into_iter()
                    .map(|(key, binding)| {
                        let item_def = self.collector.definitions.get_mut(binding.def_id);
                        item_def.petal = def_petal;
                        (key, binding)
                    })
                    .collect::<HashMap<_, _>>(),
                ),
            );

            self.collector
                .tctx
                .ext_table
                .attach_hir_ext_id(extend_item.id, ext_id);
        }

        Ok(())
    }

    pub(crate) fn resolve_struct(&mut self, struct_item: &HirItem) -> ResolveResult {
        let HirItemKind::Struct(strct) = &struct_item.kind else {
            unreachable!()
        };

        if !struct_item.is_top_level() {
            if let Err(Some(diag)) = self.collector.collect_struct(&struct_item) {
                self.collector.dctx.add(diag);
            }
        }

        let def_id = self.collector.definitions.expect_def_id(struct_item.id);
        let scope_id = self.collector.scope_ctx.expect_def_scope_id(def_id);

        self.enter_scope(scope_id, |s| {
            let Some(generic_params) = &strct.generic_params else {
                return;
            };

            for generic_param in &generic_params.list {
                for proto_req in &generic_param.proto_reqs {
                    if let Err(Some(diag)) =
                        s.expect_space(DefSpace::Type, |s| s.resolve_path(proto_req))
                    {
                        s.dctx.add(diag);
                    }
                }
            }

            for field in &strct.fields.list {
                if let Err(Some(diag)) = s.resolve_ty(&field.ty) {
                    s.dctx.add(diag);
                }
            }
        });

        Ok(())
    }

    pub(crate) fn resolve_fn(&mut self, fn_item: &HirItem) -> ResolveResult {
        let HirItemKind::Fn(func) = &fn_item.kind else {
            unreachable!()
        };

        if !fn_item.is_top_level() {
            if let Err(Some(diag)) = self.collector.collect_fn(&fn_item) {
                self.collector.dctx.add(diag);
            }
        }

        // TODO: TEMP:
        // let def_id = self.collector.definitions.get_def_id(fn_item.id).unwrap();
        let Some(def_id) = self.collector.definitions.get_def_id(fn_item.id) else {
            return Ok(());
        };

        let Some(scope_id) = self.collector.scope_ctx.get_id_from_def(def_id) else {
            bug!("no scope for def: {def_id:?}")
        };

        self.enter_scope(scope_id, |s| {
            if let Some(generic_params) = &func.sig.generic_params {
                for generic_param in &generic_params.list {
                    for proto_req in &generic_param.proto_reqs {
                        if let Err(Some(diag)) =
                            s.expect_space(DefSpace::Type, |s| s.resolve_path(proto_req))
                        {
                            s.dctx.add(diag);
                        }
                    }
                }
            }

            for param in &func.sig.params.list {
                if let Err(Some(diag)) = s.resolve_ty(&param.ty) {
                    s.dctx.add(diag);
                }
            }

            if let Some(ret_ty) = &func.sig.ret_ty {
                if let Err(Some(diag)) = s.resolve_ty(&ret_ty) {
                    s.dctx.add(diag);
                }
            }

            if let Err(Some(diag)) = s.resolve_block(&func.body) {
                s.dctx.add(diag);
            }
        });

        Ok(())
    }

    pub(crate) fn resolve_var_decl(&mut self, var_decl: &HirItem) -> ResolveResult {
        let HirItemKind::VarDecl(decl) = &var_decl.kind else {
            unreachable!()
        };

        if !var_decl.is_top_level() {
            if let Err(Some(diag)) = self.collector.collect_var(var_decl) {
                self.collector.dctx.add(diag);
            }
        }

        if let Some(ty) = decl.ty {
            if let Err(Some(diag)) = self.resolve_ty(&ty) {
                self.dctx.add(diag);
            }
        }

        if let Some(expr) = decl.val {
            if let Err(Some(diag)) = self.resolve_expr(&expr) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }
}
