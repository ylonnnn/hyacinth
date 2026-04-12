use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    def::{DefAccessibility, DefKind, Definition},
    item::{HirFnParam, HirItem, HirItemKind, HirPetalKind},
};
use hycc_scope::Scope;
use hycc_util::ternary;

use crate::collector::{CollectResult, Collector};

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
                petal_item.id,
                petal_item.span,
                petal_item.accessibility,
            );

            let def_id = ternary!(petal.is_inline(), self.try_define(def), self.define(def))?;
            let scope_id = self.scope_ctx.try_attach_to_def(def_id, Scope::new());
            self.scope_ctx.push_id(scope_id);
            scopes += 1;
        }

        // Define the items within the petal
        for item in &petal.items {
            // Manual diagnostic collection to allow for multiple
            // diagnostics from multiple item collection
            match self.collect_item(&item) {
                Ok(_) => {}
                Err(diag) => {
                    if let Some(diag) = diag {
                        self.dctx.add(diag);
                    }
                }
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

        let def_id = self.define(Definition::new(
            func.ident.ident,
            DefKind::Fn,
            fn_item.id,
            fn_item.span,
            fn_item.accessibility,
        ))?;

        let scope_id = self.scope_ctx.attach_to_def(def_id, Scope::new());
        self.enter_scope(scope_id, |s| {
            // Define the function parameters
            for param in &func.params.list {
                match s.collect_fn_param(&param) {
                    Ok(_) => {}
                    Err(diag) => {
                        if let Some(diag) = diag {
                            s.dctx.add(diag);
                        }
                    }
                }
            }
        });

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
