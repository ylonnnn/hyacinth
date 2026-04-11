use hycc_hir::{
    def::{DefKind, Definition},
    item::{HirItem, HirItemKind},
};

use crate::collector::{CollectResult, Collector};

impl<'t, 'h> Collector<'t, 'h> {
    pub(crate) fn collect_item(&mut self, item: &HirItem) -> CollectResult {
        match &item.kind {
            HirItemKind::Petal(_) => todo!("collect petals"),
            HirItemKind::Fn(_) => self.collect_fn(&item),

            HirItemKind::VarDecl(_) => self.collect_var(&item),
        }
    }

    pub(crate) fn collect_petal(&mut self, petal_item: &HirItem) -> CollectResult {
        let HirItemKind::Petal(petal) = &petal_item.kind else {
            unreachable!();
        };

        // Definition::new(
        // )

        todo!()
    }

    pub(crate) fn collect_fn(&mut self, fn_item: &HirItem) -> CollectResult {
        let HirItemKind::Fn(func) = &fn_item.kind else {
            unreachable!()
        };

        self.define(Definition::new(
            func.ident.ident,
            DefKind::Fn,
            fn_item.id,
            fn_item.span,
            fn_item.accessibility,
        ))?;

        // Create the scope of the function body
        // Define the parameters

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
