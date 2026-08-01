use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirId, HirNode,
    def::{Binding, BuiltinKind, BuiltinTyKind, DefAccessibility, DefKind, DefSpace, Definition},
    item::{HirItem, HirItemKind, HirItemLevel, HirReferTarget, HirReferTargetKind},
    scope::ScopeId,
};
use hycc_span::Span;
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
        mut lookup_scope: Option<ScopeId>,
    ) -> ResolveResult {
        match &target.kind {
            HirReferTargetKind::Child(alias) => {
                let target = target.symbol;
                let def_id = self.resolve_ident(&target, lookup_scope)?;

                let sym = target.ident.ident;
                let actual = self.collector.definitions.get(def_id);

                self.collector.scope_ctx.top_mut().define(
                    actual.kind.space(),
                    alias.unwrap_or(sym),
                    Binding::new(def_id, accessibility),
                );
            }

            HirReferTargetKind::Parent(children) => {
                lookup_scope.replace(self.expect_space(
                    DefSpace::Type,
                    |s| -> ResolveResult<ScopeId> {
                        let def_id = s.resolve_ident(&target.symbol, lookup_scope)?;
                        Ok(s.collector.scope_ctx.get_id_from_def(def_id).unwrap())
                    },
                )?);

                for child in children {
                    if let Err(Some(diag)) =
                        self.resolve_refer_target(&child, accessibility, lookup_scope)
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
        let HirItemKind::Extend(_) = &extend_item.kind else {
            unreachable!()
        };

        let ext = self.collector.ext_table.expect_hir_ext(extend_item.id);
        let HirNode::Path(target) = self.hir_table.get(ext.target) else {
            unreachable!()
        };

        let items = ext
            .items
            .iter()
            .map(|id| match self.hir_table.get(*id) {
                HirNode::Item(item) => item,
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();

        // Resolve extension target
        self.expect_space(DefSpace::Type, |s| s.resolve_path(&target))?;

        let target_def_id = self.collector.definitions.get_def_id(target.id).unwrap();
        let target_scope_id = self.collector.scope_ctx.expect_def_scope_id(target_def_id);

        let scope_id = self.collector.scope_ctx.expect_hir_scope_id(extend_item.id);

        self.enter_scope(scope_id, |s| {
            // Define `Self`
            let target_def = s.collector.definitions.get(target_def_id);
            let target_def_petal = target_def.petal;

            s.collector.scope_ctx.get_mut(scope_id).define(
                target_def.kind.space(),
                s.collector.interner.intern("Self"),
                Binding::new(target_def_id, DefAccessibility::Priv),
            );

            s.collector
                .scope_ctx
                .get_mut(scope_id)
                .redirect
                .replace(target_scope_id);

            // Pre-collection
            for item in &items {
                if let Err(Some(diag)) = s.collector.collect_item(&item) {
                    s.collector.dctx.add(diag);
                }

                s.collector.definitions.expect_mut_def(item.id).petal = target_def_petal;
            }

            s.collector.scope_ctx.get_mut(scope_id).redirect.take();

            for item in &items {
                if let Err(Some(diag)) = s.resolve_item(&item) {
                    s.dctx.add(diag);
                }
            }
        });

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

        for field in &strct.fields.list {
            if let Err(Some(diag)) = self.resolve_ty(&field.ty) {
                self.dctx.add(diag);
            }
        }

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
