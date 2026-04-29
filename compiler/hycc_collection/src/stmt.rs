use hycc_hir::stmt::{HirStmt, HirStmtKind};

use crate::collector::{CollectResult, Collector};

impl<'t, 'h> Collector<'t, 'h> {
    pub(crate) fn collect_stmt(&mut self, stmt: &HirStmt) -> CollectResult {
        match &stmt.kind {
            HirStmtKind::Ret(ret) => {
                let Some(val) = &ret.value else { return Ok(()) };
                self.collect_expr(&val)
            }

            HirStmtKind::Pass(pass) => {
                let Some(val) = &pass.value else {
                    return Ok(());
                };

                self.collect_expr(&val)
            }

            HirStmtKind::Item(item) => self.collect_item(&item),
            HirStmtKind::Expr(expr) => self.collect_expr(&expr),
        }
    }
}
