use std::sync::Arc;

use crate::{
    ResolveResult,
    diag::ResolverDiag,
    resolver_traits::{ResolveExpr, ResolveTy},
    ty::resolver::TyResolver,
};
use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    expr::{
        HirAnonFn, HirArrayExpr, HirExpr, HirExprKind, HirFnCall, HirIfExpr, HirMethodCall,
        HirStructExpr, HirTupleExpr, HirUnary,
    },
    path::HirIdentArgument,
};
use hycc_ty::ty::{InferKind, Ty};

impl<'t, 'd, 's, 'h> ResolveExpr<(), Option<ResolverDiag>> for TyResolver<'t, 'd, 's, 'h> {
    fn resolve_expr(&mut self, expr: &HirExpr) -> Result<(), Option<ResolverDiag>> {
        match &expr.kind {
            HirExprKind::Block(block) => self.resolve_block(&block),

            HirExprKind::Path(path) => self.resolve_path(&path).map(|_| ()),

            HirExprKind::RefExpr(reference) => self.resolve_expr(&reference.expr),

            HirExprKind::Literal(_) => Ok(()),

            HirExprKind::Binary(_, left, right) => self.resolve_binary_expr(&left, &right),
            HirExprKind::Unary(unary) => self.resolve_unary_expr(&unary),

            HirExprKind::Assign(assignee, expr) => self.resolve_assign_expr(&assignee, &expr),

            HirExprKind::Array(array) => self.resolve_array_expr(&array),
            HirExprKind::Tuple(tup) => self.resolve_tuple_expr(&tup),
            HirExprKind::Struct(strct) => self.resolve_struct_expr(&strct),

            HirExprKind::AnonFn(_) => self.resolve_anon_fn_expr(&expr),
            HirExprKind::FnCall(call) => self.resolve_fn_call_expr(&call),
            HirExprKind::FieldAccess(access) => self.resolve_expr(&access.leading),
            HirExprKind::MethodCall(call) => self.resolve_method_call_expr(&call),

            HirExprKind::If(ite) => self.resolve_if_expr(&ite),
        }
    }
}

impl<'t, 'd, 's, 'h> TyResolver<'t, 'd, 's, 'h> {
    pub(crate) fn resolve_binary_expr(&mut self, left: &HirExpr, right: &HirExpr) -> ResolveResult {
        if let Err(Some(diag)) = self.resolve_expr(&left) {
            self.dctx.add(diag);
        }

        self.resolve_expr(&right)
    }

    pub(crate) fn resolve_unary_expr(&mut self, unary: &HirUnary) -> ResolveResult {
        self.resolve_expr(unary.expr())
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
        for element in &array.elements {
            if let Err(Some(diag)) = self.resolve_expr(&element) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }

    pub(crate) fn resolve_tuple_expr(&mut self, tup: &HirTupleExpr) -> ResolveResult {
        for element in &tup.elements {
            if let Err(Some(diag)) = self.resolve_expr(&element) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }

    pub(crate) fn resolve_struct_expr(&mut self, strct: &HirStructExpr) -> ResolveResult {
        if let Err(Some(diag)) = self.resolve_path(&strct.path) {
            self.dctx.add(diag);
        }

        for field in &strct.fields {
            if let Err(Some(diag)) = self.resolve_expr(&field.val) {
                self.dctx.add(diag);
            }
        }

        Ok(())
    }

    pub(crate) fn resolve_anon_fn_expr(&mut self, anfn_expr: &HirExpr) -> ResolveResult {
        let HirExprKind::AnonFn(anfn) = &anfn_expr.kind else {
            unreachable!()
        };

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

        let fn_ty = self
            .tctx
            .make_fn_ty(Arc::new([]), None, params.into(), ret_ty);
        self.tctx
            .attach_to_hir(anfn_expr.id, Ty::new(fn_ty, anfn_expr.span));

        self.resolve_block(&anfn.body)
    }

    pub(crate) fn resolve_fn_call_expr(&mut self, call: &HirFnCall) -> ResolveResult {
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

    pub(crate) fn resolve_method_call_expr(&mut self, call: &HirMethodCall) -> ResolveResult {
        if let Err(Some(diag)) = self.resolve_expr(&call.receiver) {
            self.dctx.add(diag);
        }

        if let Some(arguments) = &call.callee.arguments {
            for argument in &arguments.data {
                match argument {
                    HirIdentArgument::Ty(ty) => {
                        self.resolve_ty(ty)?;
                    }
                    HirIdentArgument::Expr(_expr) => {
                        // todo: const generics -> GenericArg::Const
                        todo!("const generic args");
                    }
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
