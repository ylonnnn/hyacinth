use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    def::DefSpace,
    expr::{HirExpr, HirExprKind, HirUnary},
};

use crate::{ResolveResult, ident::resolver::Resolver};

impl<'c> Resolver<'c> {
    pub(crate) fn resolve_expr(&mut self, expr: &HirExpr) -> ResolveResult {
        self.expect_space(DefSpace::Value, |s| match &expr.kind {
            HirExprKind::Path(path) => s.resolve_path(&path),
            HirExprKind::RefExpr(reference) => s.resolve_expr(&reference.expr),

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

            HirExprKind::Block(block) => s.resolve_block(&block),

            HirExprKind::Array(array) => Ok(for expr in &array.elements {
                if let Err(Some(diag)) = s.resolve_expr(&expr) {
                    s.dctx.add(diag);
                }
            }),

            HirExprKind::Tuple(tup) => Ok(for el in &tup.elements {
                if let Err(Some(diag)) = s.resolve_expr(&el) {
                    s.dctx.add(diag);
                }
            }),

            HirExprKind::Struct(strct) => {
                s.expect_space(DefSpace::Type, |s| {
                    if let Err(Some(diag)) = s.resolve_path(&strct.path) {
                        s.dctx.add(diag);
                    }
                });

                Ok(for field in &strct.fields {
                    if let Err(Some(diag)) = s.resolve_expr(&field.val) {
                        s.dctx.add(diag);
                    }
                })
            }

            HirExprKind::AnonFn(anfn) => {
                let Some(scope_id) = s.collector.scope_ctx.get_id(expr.id) else {
                    return Ok(());
                };

                s.enter_scope(scope_id, |s| {
                    for param in &anfn.params.list {
                        let Some(p_ty) = param.ty else {
                            continue;
                        };

                        if let Err(Some(diag)) = s.resolve_ty(&p_ty) {
                            s.dctx.add(diag);
                        }
                    }

                    if let Some(ret_ty) = &anfn.ret_ty {
                        if let Err(Some(diag)) = s.resolve_ty(&ret_ty) {
                            s.dctx.add(diag);
                        }
                    }

                    s.resolve_block(&anfn.body)
                })
            }

            HirExprKind::FnCall(call) => {
                if let Err(Some(diag)) = s.resolve_expr(&call.callee) {
                    s.dctx.add(diag);
                }

                Ok(for argument in &call.arguments.data {
                    if let Err(Some(diag)) = s.resolve_expr(&argument) {
                        s.dctx.add(diag);
                    }
                })
            }

            HirExprKind::FieldAccess(access) => {
                Ok(if let Err(Some(diag)) = s.resolve_expr(&access.leading) {
                    s.dctx.add(diag);
                })
            }

            HirExprKind::MethodCall(call) => {
                if let Err(Some(diag)) = s.resolve_expr(&call.receiver) {
                    s.dctx.add(diag);
                }

                Ok(for argument in &call.arguments.data {
                    if let Err(Some(diag)) = s.resolve_expr(&argument) {
                        s.dctx.add(diag);
                    }
                })
            }
        })
    }
}
