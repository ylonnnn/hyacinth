use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    def::{DefAccessibility, DefKind, DefSpace, Definition},
    expr::{
        HirArrayExpr, HirExpr, HirExprKind, HirFnCall, HirIfExpr, HirMethodCall, HirStructExpr,
        HirTupleExpr, HirUnary,
    },
    path::HirIdentArgument,
    scope::Scope,
};

use crate::{ResolveResult, ident::resolver::Resolver};

impl<'c, 'i, 'h> Resolver<'c, 'i, 'h> {
    pub(crate) fn resolve_expr(&mut self, expr: &HirExpr) -> ResolveResult {
        self.expect_space(DefSpace::Value, |s| match &expr.kind {
            HirExprKind::Path(path) => s.resolve_path(&path),
            HirExprKind::RefExpr(reference) => s.resolve_expr(&reference.expr),

            HirExprKind::Literal(_) => Ok(()),

            HirExprKind::Binary(_, left, right) => s.resolve_binary_expr(&left, &right),
            HirExprKind::Unary(unary) => s.resolve_expr(unary.expr()),

            HirExprKind::Assign(assignee, expr) => s.resolve_assign_expr(&assignee, &expr),

            HirExprKind::Block(block) => s.resolve_block(&block),

            HirExprKind::Array(array) => s.resolve_array_expr(&array),
            HirExprKind::Tuple(tup) => s.resolve_tuple_expr(&tup),
            HirExprKind::Struct(strct) => s.resolve_struct_expr(&strct),
            HirExprKind::AnonFn(anfn) => s.resolve_anon_fn_expr(&expr),

            HirExprKind::FnCall(call) => s.resolve_fn_call_expr(&call),
            HirExprKind::FieldAccess(access) => s.resolve_expr(access.leading),
            HirExprKind::MethodCall(call) => s.resolve_method_call_expr(&call),

            HirExprKind::If(ite) => s.resolve_if_expr(&ite),
        })
    }

    pub(crate) fn resolve_binary_expr(&mut self, left: &HirExpr, right: &HirExpr) -> ResolveResult {
        if let Err(Some(diag)) = self.resolve_expr(&left) {
            self.dctx.add(diag);
        }

        self.resolve_expr(&right)
    }

    pub(crate) fn resolve_assign_expr(
        &mut self,
        assignee: &HirExpr,
        expr: &HirExpr,
    ) -> ResolveResult {
        if let Err(Some(diag)) = self.resolve_expr(&assignee) {
            self.dctx.add(diag);
        }

        self.resolve_expr(&expr)
    }

    pub(crate) fn resolve_array_expr(&mut self, array: &HirArrayExpr) -> ResolveResult {
        for expr in &array.elements {
            if let Err(Some(diag)) = self.resolve_expr(&expr) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }

    pub(crate) fn resolve_tuple_expr(&mut self, tup: &HirTupleExpr) -> ResolveResult {
        for el in &tup.elements {
            if let Err(Some(diag)) = self.resolve_expr(&el) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }

    pub(crate) fn resolve_struct_expr(&mut self, strct: &HirStructExpr) -> ResolveResult {
        self.expect_space(DefSpace::Type, |s| {
            if let Err(Some(diag)) = s.resolve_path(&strct.path) {
                s.dctx.add(diag);
            }
        });

        Ok(for field in &strct.fields {
            if let Err(Some(diag)) = self.resolve_expr(&field.val) {
                self.dctx.add(diag);
            }
        })
    }

    pub(crate) fn resolve_anon_fn_expr(&mut self, anfn_expr: &HirExpr) -> ResolveResult {
        let HirExprKind::AnonFn(anfn) = &anfn_expr.kind else {
            unreachable!()
        };

        let scope_id = self.collector.scope_ctx.attach(anfn_expr.id, Scope::new());

        self.enter_scope(scope_id, |s| {
            for param in &anfn.params.list {
                if let Err(Some(diag)) = s.collector.define(Definition::new(
                    param.ident.ident,
                    DefKind::FnParam,
                    Some(s.collector.petal_ctx.top_id()),
                    param.id,
                    param.span,
                    DefAccessibility::Priv,
                )) {
                    s.collector.dctx.add(diag);
                }

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

    pub(crate) fn resolve_fn_call_expr(&mut self, call: &HirFnCall) -> ResolveResult {
        if let Err(Some(diag)) = self.resolve_expr(&call.callee) {
            self.dctx.add(diag);
        }

        Ok(for argument in &call.arguments.data {
            if let Err(Some(diag)) = self.resolve_expr(&argument) {
                self.dctx.add(diag);
            }
        })
    }

    pub(crate) fn resolve_method_call_expr(&mut self, call: &HirMethodCall) -> ResolveResult {
        if let Err(Some(diag)) = self.resolve_expr(&call.receiver) {
            self.dctx.add(diag);
        }

        if let Some(arguments) = &call.callee.arguments {
            for argument in &arguments.data {
                let res = match &argument {
                    HirIdentArgument::Expr(expr) => {
                        self.expect_space(DefSpace::Value, |s| s.resolve_expr(&expr))
                    }

                    HirIdentArgument::Ty(ty) => {
                        self.expect_space(DefSpace::Type, |s| s.resolve_ty(&ty))
                    }
                };

                if let Err(Some(diag)) = res {
                    self.dctx.add(diag);
                }
            }
        }

        for argument in &call.arguments.data {
            if let Err(Some(diag)) = self.resolve_expr(&argument) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }

    pub(crate) fn resolve_if_expr(&mut self, ite: &HirIfExpr) -> ResolveResult {
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
