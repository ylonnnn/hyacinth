use hycc_hir::{
    def::{DefKind, Definition},
    item::{HirItem, HirItemKind},
};

use crate::collector::{CollectResult, Collector};

impl<'t, 'h> Collector<'t, 'h> {
    pub(crate) fn collect_item(&mut self, item: &HirItem) -> CollectResult {
        let definition = match &item.kind {
            HirItemKind::Fn(func) => {
                Definition::new_default(func.ident.ident, DefKind::Fn, item.id, item.span)
            }

            HirItemKind::VarDecl(decl) => {
                Definition::new_default(decl.ident.ident, DefKind::Var, item.id, item.span)
            }
        };

        self.define(definition)?;

        Ok(())
    }
}
