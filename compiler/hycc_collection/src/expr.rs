use hycc_diagnostic::DiagnosticContext;
use hycc_hir::expr::{HirExpr, HirExprKind, HirUnary};

use crate::collector::{CollectResult, Collector};

impl<'t, 'h> Collector<'t, 'h> {
    pub(crate) fn collect_expr(&mut self, expr: &HirExpr) -> CollectResult {
        match &expr.kind {
            HirExprKind::Block(block) => self.collect_block(&block),

            HirExprKind::Path(_) => Ok(()),
            HirExprKind::RefExpr(reference) => self.collect_expr(&reference.expr),

            HirExprKind::Literal(_) => Ok(()),

            HirExprKind::Binary(_, left, right) => {
                if let Err(Some(diag)) = self.collect_expr(&left) {
                    self.dctx.add(diag);
                }

                self.collect_expr(&right)
            }

            HirExprKind::Unary(unary) => match unary.as_ref() {
                HirUnary::Pre(_, expr) | HirUnary::Post(_, expr) => self.collect_expr(&expr),
            },

            HirExprKind::Assign(assignee, expr) => {
                if let Err(Some(diag)) = self.collect_expr(&assignee) {
                    self.dctx.add(diag);
                }

                self.collect_expr(&expr)
            }

            HirExprKind::Array(array) => {
                for element in &array.elements {
                    if let Err(Some(diag)) = self.collect_expr(&element) {
                        self.dctx.add(diag);
                    }
                }

                Ok(())
            }

            HirExprKind::Tuple(tup) => {
                for element in &tup.elements {
                    if let Err(Some(diag)) = self.collect_expr(&element) {
                        self.dctx.add(diag);
                    }
                }

                Ok(())
            }

            HirExprKind::Struct(strct) => {
                for field in &strct.fields {
                    if let Err(Some(diag)) = self.collect_expr(&field.val) {
                        self.dctx.add(diag);
                    }
                }

                Ok(())
            }

            HirExprKind::FnCall(call) => {
                if let Err(Some(diag)) = self.collect_expr(&call.callee) {
                    self.dctx.add(diag);
                }

                for argument in &call.arguments.data {
                    if let Err(Some(diag)) = self.collect_expr(&argument) {
                        self.dctx.add(diag);
                    }
                }

                Ok(())
            }

            HirExprKind::FieldAccess(access) => self.collect_expr(&access.leading),
            HirExprKind::MethodCall(call) => {
                if let Err(Some(diag)) = self.collect_expr(&call.receiver) {
                    self.dctx.add(diag);
                }

                for argument in &call.arguments.data {
                    if let Err(Some(diag)) = self.collect_expr(&argument) {
                        self.dctx.add(diag);
                    }
                }

                Ok(())
            }
        }
    }
}
