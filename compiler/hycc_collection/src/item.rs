use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    def::{DefAccessibility, DefKind, Definition},
    item::{HirFnParam, HirItem, HirItemKind, HirPetalKind},
};
use hycc_scope::Scope;
use hycc_util::ternary;

use crate::collector::{CollectResult, CollectionLevel, Collector};

impl<'t, 'h> Collector<'t, 'h> {
    pub(crate) fn collect_item(&mut self, item: &HirItem) -> CollectResult {
        match &item.kind {
            HirItemKind::Petal(_) => self.collect_petal(&item),
            HirItemKind::Fn(_) => self.collect_fn(&item),

            HirItemKind::VarDecl(_) => self.collect_var(&item),
        }
    }

    pub(crate) fn collect_petal(&mut self, petal_item: &HirItem) -> CollectResult {
        let HirItemKind::Petal(petal) = &petal_item.kind else {
            unreachable!();
        };

        if matches!(petal.kind, HirPetalKind::Root) {
            panic!("root petals cannot be collected!")
        }

        let path = match &petal.kind {
            HirPetalKind::File(path) | HirPetalKind::Inline(path) => path,
            _ => unreachable!(),
        };

        let mut scopes = 0;
        for segment in &path.segments {
            let def = Definition::new(
                segment.ident.ident,
                DefKind::Petal,
                segment.id,
                petal_item.span,
                petal_item.accessibility,
            );

            let def_id = ternary!(
                self.is_expected_to_be_collected(),
                {
                    let def_id = self.definitions.get_def_id(segment.id);
                    if let Some(def_id) = def_id {
                        *def_id
                    } else {
                        return Ok(());
                    }
                },
                ternary!(petal.is_inline(), self.try_define(def), self.define(def))?
            );
            let scope_id = self.scope_ctx.try_attach_to_def(def_id, Scope::new());

            self.definitions.define_id_hir(segment.id, def_id);
            self.scope_ctx.push_id(scope_id);

            scopes += 1;
        }

        for item in &petal.items {
            if let Err(Some(diag)) = self.collect_item(&item) {
                self.dctx.add(diag);
            }
        }

        while scopes > 0 {
            self.scope_ctx.pop();
            scopes -= 1;
        }

        Ok(())
    }

    pub(crate) fn collect_fn(&mut self, fn_item: &HirItem) -> CollectResult {
        let HirItemKind::Fn(func) = &fn_item.kind else {
            unreachable!()
        };

        let def_id = ternary!(
            self.is_expected_to_be_collected(),
            {
                let def_id = self.definitions.get_def_id(fn_item.id);
                if let Some(def_id) = def_id {
                    *def_id
                } else {
                    return Ok(());
                }
            },
            self.define(Definition::new(
                func.ident.ident,
                DefKind::Fn,
                fn_item.id,
                fn_item.span,
                fn_item.accessibility,
            ))?
        );

        let scope_id = self.scope_ctx.try_attach_to_def(def_id, Scope::new());
        let prev_n_lev = self.node_level;

        self.node_level = CollectionLevel::Local;

        self.enter_scope(scope_id, |s| {
            match s.level {
                CollectionLevel::Top => {
                    // Define the function parameters
                    for param in &func.params.list {
                        if let Err(Some(diag)) = s.collect_fn_param(&param) {
                            s.dctx.add(diag);
                        }
                    }
                }

                CollectionLevel::Local => {
                    for stmt in &func.body.stmts {
                        if let Err(Some(diag)) = s.collect_stmt(&stmt) {
                            s.dctx.add(diag);
                        }
                    }
                }
            }
        });

        self.node_level = prev_n_lev;

        Ok(())
    }

    pub(crate) fn collect_fn_param(&mut self, param: &HirFnParam) -> CollectResult {
        self.define(Definition::new(
            param.ident.ident,
            DefKind::FnParam,
            param.id,
            param.span,
            DefAccessibility::Priv,
        ))?;

        Ok(())
    }

    pub(crate) fn collect_var(&mut self, var_item: &HirItem) -> CollectResult {
        let HirItemKind::VarDecl(var) = &var_item.kind else {
            unreachable!()
        };

        self.define(Definition::new(
            var.ident.ident,
            DefKind::Var,
            var_item.id,
            var_item.span,
            var_item.accessibility,
        ))?;

        Ok(())
    }
}
