use crate::{ResolveResult, ty::resolver::TyResolver};
use hycc_diagnostic::DiagnosticContext;
use hycc_hir::expr::{HirExpr, HirExprKind, HirUnary};

impl<'d, 'r> TyResolver<'d, 'r> {
    pub(crate) fn resolve_expr(&mut self, expr: &HirExpr) -> ResolveResult {
        match &expr.kind {
            HirExprKind::Block(block) => self.resolve_block(&block),

            HirExprKind::Path(path) => Ok(()), // TODO: try to resolve expression arguments
            HirExprKind::RefExpr(reference) => self.resolve_expr(&reference.expr),

            HirExprKind::Literal(_) => Ok(()),

            HirExprKind::Binary(_, left, right) => {
                if let Err(Some(diag)) = self.resolve_expr(&left) {
                    self.dctx.add(diag);
                }

                self.resolve_expr(&right)
            }

            HirExprKind::Unary(unary) => match unary.as_ref() {
                HirUnary::Pre(_, expr) | HirUnary::Post(_, expr) => self.resolve_expr(&expr),
            },

            HirExprKind::Assign(assignee, expr) => {
                if let Err(Some(diag)) = self.resolve_expr(&assignee) {
                    self.dctx.add(diag);
                }

                self.resolve_expr(&expr)
            }

            HirExprKind::Array(array) => {
                for element in &array.elements {
                    if let Err(Some(diag)) = self.resolve_expr(&element) {
                        self.dctx.add(diag);
                    }
                }

                Ok(())
            }

            HirExprKind::Tuple(tup) => {
                for element in &tup.elements {
                    if let Err(Some(diag)) = self.resolve_expr(&element) {
                        self.dctx.add(diag);
                    }
                }

                Ok(())
            }

            HirExprKind::Struct(strct) => {
                for field in &strct.fields {
                    if let Err(Some(diag)) = self.resolve_expr(&field.val) {
                        self.dctx.add(diag);
                    }
                }

                Ok(())
            }

            HirExprKind::FnCall(call) => {
                if let Err(Some(diag)) = self.resolve_expr(&call.callee) {
                    self.dctx.add(diag);
                }

                for argument in &call.arguments.data {
                    if let Err(Some(diag)) = self.resolve_expr(&argument) {
                        self.dctx.add(diag);
                    }
                }

                Ok(())
            }

            HirExprKind::FieldAccess(access) => self.resolve_expr(&access.leading),
            HirExprKind::MethodCall(call) => {
                if let Err(Some(diag)) = self.resolve_expr(&call.receiver) {
                    self.dctx.add(diag);
                }

                for argument in &call.arguments.data {
                    if let Err(Some(diag)) = self.resolve_expr(&argument) {
                        self.dctx.add(diag);
                    }
                }

                Ok(())
            }
        }
    }
}
