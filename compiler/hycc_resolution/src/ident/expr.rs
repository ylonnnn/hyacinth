use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    def::DefSpace,
    expr::{HirExpr, HirExprKind, HirUnary},
};

use crate::{ResolveResult, ident::resolver::Resolver};

impl<'s, 'd> Resolver<'s, 'd> {
    pub(crate) fn resolve_expr(&mut self, expr: &HirExpr) -> ResolveResult {
        self.expect_space(DefSpace::Value, |s| match &expr.kind {
            HirExprKind::Path(path) => s.resolve_path(&path),
            HirExprKind::Literal(_) => Ok(()),
            HirExprKind::Binary(_, left, right) => {
                if let Err(Some(diag)) = s.resolve_expr(&left) {
                    s.dctx.add(diag);
                }

                s.resolve_expr(&right)
            }
            HirExprKind::Unary(unary) => match unary.as_ref() {
                HirUnary::Pre(_, expr) | HirUnary::Post(_, expr) => s.resolve_expr(&expr),
            },
            HirExprKind::Assign(assignee, expr) => {
                if let Err(Some(diag)) = s.resolve_expr(&assignee) {
                    s.dctx.add(diag);
                }

                s.resolve_expr(&expr)
            }

            HirExprKind::Array(array) => {
                for expr in &array.elements {
                    if let Err(Some(diag)) = s.resolve_expr(&expr) {
                        s.dctx.add(diag);
                    }
                }

                Ok(())
            }
        })
    }
}
