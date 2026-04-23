use hycc_diagnostic::DiagnosticContext;
use hycc_hir::expr::{HirArrayExpr, HirExpr, HirExprKind, HirLiteral};
use hycc_ty::{context::TyId, ty::InferKind};

use crate::{
    diag::{InferDiag, InferDiagErrorKind},
    inferer::{InferResult, TyInferer},
};

impl<'t, 'd, 'r> TyInferer<'t, 'd, 'r> {
    pub(crate) fn infer_expr(&mut self, expr: &HirExpr) -> InferResult<TyId> {
        match &expr.kind {
            HirExprKind::Path(path) => self.infer_path(&path),
            HirExprKind::Literal(lit) => self.infer_literal(&lit),
            HirExprKind::Binary(op, left, right) => todo!("infer binary"),
            HirExprKind::Unary(unary) => todo!("infer unary"),
            HirExprKind::Assign(assignee, expr) => todo!("infer assignment"),
            HirExprKind::Array(array) => self.infer_array_expr(&array),
        }
    }

    pub(crate) fn infer_literal(&mut self, lit: &HirLiteral) -> InferResult<TyId> {
        Ok(match &lit {
            HirLiteral::Int { .. } => self.tctx.make_inferred_ty(InferKind::Int),
            HirLiteral::Float(_) => self.tctx.make_inferred_ty(InferKind::Float),
            HirLiteral::Bool(_) => self.tctx.make_bool_ty(),
            HirLiteral::Char(_) => self.tctx.make_char_ty(),
            HirLiteral::String(_) => self.tctx.make_string_ty(),
        })
    }

    pub(crate) fn infer_array_expr(&mut self, array: &HirArrayExpr) -> InferResult<TyId> {
        let mut el_ty_id = self.tctx.make_inferred_ty(InferKind::Any);

        for expr in &array.elements {
            let curr_el_ty_id = match self.infer_expr(&expr) {
                Ok(ty_id) => ty_id,
                Err(diag) => {
                    if let Some(diag) = diag {
                        self.dctx.add(diag);
                    }

                    continue;
                }
            };

            if !self.tctx.unify_ty(el_ty_id, curr_el_ty_id) {
                self.dctx.add(InferDiag::error(
                    expr.span,
                    InferDiagErrorKind::TypeMismatch {
                        expected: el_ty_id,
                        received: curr_el_ty_id,
                    },
                ));
            }

            el_ty_id = self.tctx.resolve_ty(el_ty_id);
            self.tctx.resolve_ty(curr_el_ty_id);
        }

        Ok(self.tctx.make_array_ty(el_ty_id))
    }
}
