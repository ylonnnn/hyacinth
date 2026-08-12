use hycc_hir::stmt::{HirStmt, HirStmtKind};

use crate::{ResolveResult, resolver_traits::ResolveExpr, ty::resolver::TyResolver};

impl<'t, 'd, 's, 'h> TyResolver<'t, 'd, 's, 'h> {
    pub(crate) fn resolve_stmt(&mut self, stmt: &HirStmt) -> ResolveResult {
        match &stmt.kind {
            HirStmtKind::Ret(ret) => {
                let Some(val) = &ret.value else { return Ok(()) };
                self.resolve_expr(&val)
            }

            HirStmtKind::Pass(pass) => {
                let Some(val) = &pass.value else {
                    return Ok(());
                };

                self.resolve_expr(&val)
            }

            HirStmtKind::Item(item) => self.resolve_item(&item),
            HirStmtKind::Expr(expr) => self.resolve_expr(&expr),
        }
    }
}
