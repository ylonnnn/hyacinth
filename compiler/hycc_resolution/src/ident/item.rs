use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    def::{Binding, DefAccessibility, DefKind, DefSpace, Definition},
    item::{HirItem, HirItemKind, HirPetalKind, HirReferTarget, HirReferTargetKind},
    scope::{Scope, ScopeId},
};
use hycc_util::{bug, ternary};

use crate::{
    ResolveResult,
    diag::{ResolverDiag, ResolverDiagErrorKind},
    ident::resolver::Resolver,
};

impl<'c, 'i> Resolver<'c, 'i> {
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

        //     let HirItemKind::Petal(petal) = &petal_item.kind else {
        //         unreachable!()
        //     };

        //     // if let Err(Some(diag)) = self.collector.collect_petal(&petal_item) {
        //     //     self.collector.dctx.add(diag);
        //     // }

        //     let path = match &petal.kind {
        //         HirPetalKind::File(path) | HirPetalKind::Inline(path) => path,
        //         _ => unreachable!(),
        //     };

        //     let mut pushed: usize = 0;
        //     for segment in &path.segments {
        //         let result = if let Some(def_id) = self.collector.definitions.get_def_id(segment.id) {
        //             Ok((*def_id, true))
        //         } else {
        //             let def = Definition::new(
        //                 segment.ident.ident,
        //                 DefKind::Petal,
        //                 Some(self.collector.petal_ctx.top_id()),
        //                 segment.id,
        //                 petal_item.span,
        //                 petal_item.accessibility,
        //             );

        //             ternary!(
        //                 petal.is_inline(),
        //                 self.collector.try_define(def),
        //                 self.collector.define(def).map(|def_id| (def_id, false))
        //             )
        //         };

        //         let (def_id, defined) = match result {
        //             Ok(res) => res,
        //             Err(diag) => {
        //                 diag.map(|diag| self.collector.dctx.add(diag));
        //                 break;
        //             }
        //         };

        //         self.collector.definitions.define_id_hir(segment.id, def_id);

        //         let petal_id = self.collector.petal_ctx.try_create_child_petal(def_id);
        //         self.collector.petal_ctx.push(petal_id);

        //         let scope_id = self
        //             .collector
        //             .scope_ctx
        //             .try_attach_to_def(def_id, Scope::new());
        //         self.collector.scope_ctx.push_id(scope_id);

        //         pushed += 1;

        //         if !defined {
        //             self.collector.define_spathe_at_current_petal();
        //             self.collector.define_super_at_current_petal();
        //         }

        //         // let Some((def_id, _)) = self.get_def_id(Some(DefSpace::Type), segment.ident.ident)
        //         // else {
        //         //     bug!("no def_id for ident: {:?}", segment.ident.ident)
        //         // };

        //         // self.collector
        //         //     .petal_ctx
        //         //     .push(match self.collector.petal_ctx.get_id_by_def(def_id) {
        //         //         Some(petal_id) => petal_id,
        //         //         _ => bug!("no petal attached to def id {def_id:?}"),
        //         //     });

        //         // let scope_id = self.collector.scope_ctx.get_id_from_def(def_id).unwrap();
        //         // self.collector.scope_ctx.push_id(scope_id);

        //         // pushed += 1;
        //     }

        //     let result = f(self);

        //     for _ in 0..pushed {
        //         self.collector.scope_ctx.pop();
        //         self.collector.petal_ctx.pop();
        //     }

        //     Ok(result)
    }

    pub(crate) fn resolve_item(&mut self, item: &HirItem) -> ResolveResult {
        match &item.kind {
            HirItemKind::Refer(_) => self.resolve_refer(&item),
            HirItemKind::Petal(_) => self.resolve_petal(&item),
            HirItemKind::Struct(_) => self.resolve_struct(&item),
            HirItemKind::Fn(_) => self.resolve_fn(&item),
            HirItemKind::VarDecl(_) => self.resolve_var_decl(&item),
        }
    }

    pub(crate) fn resolve_refer(&mut self, refer_item: &HirItem) -> ResolveResult {
        let HirItemKind::Refer(refer) = &refer_item.kind else {
            unreachable!();
        };

        dbg!(refer_item.accessibility);
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

                // self.collector.scope_ctx.top_mut().define(
                //     actual.kind.space(),
                //     alias.unwrap_or(sym),
                //     def_id,
                // );

                self.collector.scope_ctx.top_mut().define(
                    actual.kind.space(),
                    alias.unwrap_or(sym),
                    Binding::new(def_id, accessibility),
                );

                // if let Err(Some(diag)) = self.collector.define(Definition::new(
                //     alias.unwrap_or(sym),
                //     DefKind::Refer(Box::new(actual.kind.clone()), def_id),
                //     actual.petal,
                //     actual.hir_id,
                //     target.span,
                //     actual.accessibility, /* TODO: use the accessibility of the refer item */
                // )) {
                //     self.collector.dctx.add(diag);
                // }
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

    pub(crate) fn resolve_struct(&mut self, struct_item: &HirItem) -> ResolveResult {
        let HirItemKind::Struct(strct) = &struct_item.kind else {
            unreachable!()
        };

        if let Err(Some(diag)) = self.collector.collect_struct(&struct_item) {
            self.collector.dctx.add(diag);
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

        if let Err(Some(diag)) = self.collector.collect_fn(&fn_item) {
            self.collector.dctx.add(diag);
        }

        let Some((def_id, _)) = self.get_def_id(Some(DefSpace::Value), func.ident.ident) else {
            bug!("no def_id for ident: {:?}", func.ident.ident)
        };

        let Some(scope_id) = self.collector.scope_ctx.get_id_from_def(def_id) else {
            bug!("no scope for def: {def_id:?}")
        };

        self.enter_scope(scope_id, |s| {
            for param in &func.params.list {
                if let Err(Some(diag)) = s.resolve_ty(&param.ty) {
                    s.dctx.add(diag);
                }
            }

            if let Some(ret_ty) = &func.ret_ty {
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

        if let Err(Some(diag)) = self.collector.collect_var(var_decl) {
            self.collector.dctx.add(diag);
        }

        Ok(())
    }
}
