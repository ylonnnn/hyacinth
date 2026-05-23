use hycc_diagnostic::DiagnosticContext;
use hycc_hir::stmt::{HirStmt, HirStmtKind};

use crate::{ResolveResult, ty::resolver::TyResolver};

impl<'d> TyResolver<'d> {
    pub(crate) fn resolve_stmt(&mut self, stmt: &HirStmt) -> ResolveResult {
        match &stmt.kind {
            HirStmtKind::If(ite) => {
                if let Err(Some(diag)) = self.resolve_expr(&ite.cond) {
                    self.dctx.add(diag);
                }

                if let Err(Some(diag)) = self.resolve_block(&ite.consequent) {
                    self.dctx.add(diag);
                }

                ite.alternate.as_ref().map(|alt| {
                    if let Err(Some(diag)) = self.resolve_block(&alt) {
                        self.dctx.add(diag);
                    }
                });

                Ok(())
            }

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
