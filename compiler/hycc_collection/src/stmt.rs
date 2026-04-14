use hycc_hir::stmt::{HirStmt, HirStmtKind};

use crate::collector::{CollectResult, Collector};

impl<'t, 'h> Collector<'t, 'h> {
    pub(crate) fn collect_stmt(&mut self, stmt: &HirStmt) -> CollectResult {
        match &stmt.kind {
            HirStmtKind::Item(item) => self.collect_item(&item),
            _ => Ok(()),
        }
    }
}
