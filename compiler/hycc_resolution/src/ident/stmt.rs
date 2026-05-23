use hycc_diagnostic::DiagnosticContext;
use hycc_hir::stmt::{HirStmt, HirStmtKind};
use hycc_util::ternary;

use crate::{ResolveResult, ident::resolver::Resolver};

impl<'c> Resolver<'c> {
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

            HirStmtKind::Ret(ret) => ternary!(
                ret.value.is_some(),
                self.resolve_expr(&ret.value.unwrap()),
                Ok(())
            ),
            HirStmtKind::Pass(pass) => {
                // TODO: resolve block label used in pass
                ternary!(
                    pass.value.is_some(),
                    self.resolve_expr(&pass.value.unwrap()),
                    Ok(())
                )
            }

            HirStmtKind::Item(item) => self.resolve_item(&item),
            HirStmtKind::Expr(expr) => self.resolve_expr(&expr),
        }
    }
}
