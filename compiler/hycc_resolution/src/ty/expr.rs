use crate::{ResolveResult, ty::resolver::TyResolver};
use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    expr::{HirExpr, HirExprKind, HirUnary},
    path::HirIdentArgument,
};
use hycc_ty::ty::{InferKind, Ty};

impl<'t, 'd, 's> TyResolver<'t, 'd, 's> {
    pub(crate) fn resolve_expr(&mut self, expr: &HirExpr) -> ResolveResult {
        match &expr.kind {
            HirExprKind::Block(block) => self.resolve_block(&block),

            HirExprKind::Path(path) => {
                for segment in &path.segments {
                    let Some(arguments) = &segment.arguments else {
                        break;
                    };

                    for argument in &arguments.data {
                        let result = match &argument {
                            HirIdentArgument::Ty(ty) => self.resolve_ty(&ty).map(|_| ()),
                            HirIdentArgument::Expr(expr) => self.resolve_expr(&expr),
                        };

                        if let Err(Some(diag)) = result {
                            self.dctx.add(diag);
                        }
                    }
                }

                Ok(())
            }

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

            HirExprKind::AnonFn(anfn) => {
                let mut params = Vec::new();
                for param in &anfn.params.list {
                    let ty_id = if let Some(p_ty) = &param.ty {
                        match self.resolve_ty(&p_ty) {
                            Ok(ty_id) => ty_id,
                            Err(diag) => {
                                diag.map(|diag| self.dctx.add(diag));
                                continue;
                            }
                        }
                    } else {
                        self.tctx.make_inferred_ty(InferKind::Any)
                    };

                    params.push(ty_id);
                    self.tctx.attach_to_hir(
                        param.id,
                        Ty::new(ty_id, param.ty.map(|ty| ty.span).unwrap_or(param.span)),
                    );
                }

                let mut ret_ty = self.tctx.make_inferred_ty(InferKind::Any);
                // let mut ret_ty = self.tctx.make_unit_ty();
                if let Some(r_ty) = &anfn.ret_ty {
                    match self.resolve_ty(&r_ty) {
                        Ok(ty_id) => ret_ty = ty_id,
                        Err(diag) => {
                            diag.map(|diag| self.dctx.add(diag));
                        }
                    }
                }

                let fn_ty = self.tctx.make_fn_ty(params.into(), ret_ty);
                self.tctx.attach_to_hir(expr.id, Ty::new(fn_ty, expr.span));

                self.resolve_block(&anfn.body)
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

            HirExprKind::If(ite) => {
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
        }
    }
}
