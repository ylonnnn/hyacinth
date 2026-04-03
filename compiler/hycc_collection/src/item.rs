use hycc_hir::{
    def::{DefKind, Definition},
    item::{HirItem, HirItemKind},
};

use crate::collector::{CollectResult, Collector};

impl<'d> Collector<'d> {
    pub(crate) fn collect_item(&mut self, item: &HirItem) -> CollectResult {
        let definition = match &item.kind {
            HirItemKind::Fn(func) => {
                Definition::new_default(func.ident.ident, DefKind::Fn, item.id, func.span)
            }

            HirItemKind::VarDecl(_decl) => todo!("collect var decl"),
        };

        self.define(definition)?;

        Ok(())
    }
}
