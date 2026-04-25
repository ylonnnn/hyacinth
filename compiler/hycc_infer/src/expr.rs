use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirMutability, HirNode,
    def::DefKind,
    expr::{
        HirArrayExpr, HirExpr, HirExprKind, HirLiteral, HirRefExpr, HirStructExpr,
        HirStructExprField,
    },
};
use hycc_span::Span;
use hycc_ty::{
    context::TyId,
    ty::{InferKind, RefMutability},
};
use hycc_util::ternary;

use crate::{
    diag::{InferDiag, InferDiagErrorKind},
    inferer::{InferResult, TyInferer},
};

impl<'t, 'd, 'r, 'h> TyInferer<'t, 'd, 'r, 'h> {
    pub(crate) fn infer_expr(&mut self, expr: &HirExpr) -> InferResult<TyId> {
        match &expr.kind {
            HirExprKind::Path(path) => self.infer_path(&path),
            HirExprKind::RefExpr(reference) => self.infer_ref_expr(&reference),
            HirExprKind::Literal(lit) => self.infer_literal(&lit),
            HirExprKind::Binary(op, left, right) => todo!("infer binary"),
            HirExprKind::Unary(unary) => todo!("infer unary"),
            HirExprKind::Assign(assignee, expr) => todo!("infer assignment"),
            HirExprKind::Array(array) => self.infer_array_expr(&array),
            HirExprKind::Struct(strct) => self.infer_struct_expr(&strct),
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

    pub(crate) fn infer_ref_expr(&mut self, reference: &HirRefExpr) -> InferResult<TyId> {
        let inner_ty = self.infer_expr(&reference.expr)?;
        let mutability = ternary!(
            reference.mutability == HirMutability::Mutable,
            RefMutability::Mutable,
            RefMutability::Immutable
        );

        Ok(self.tctx.make_ref_ty(inner_ty, mutability))
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
                        ann_span: Span::default(),
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

    pub(crate) fn infer_struct_expr(&mut self, strct: &HirStructExpr) -> InferResult<TyId> {
        let Some(def_id) = self.resolved.get(&strct.path.id) else {
            unreachable!()
        };

        let def = self.definitions.get(*def_id);
        let DefKind::Struct(strct_def) = &def.kind else {
            return Err(Some(InferDiag::error(
                strct.path.span,
                InferDiagErrorKind::InvalidNonStructInstantiation {
                    name: def.name,
                    def_id: *def_id,
                },
            )));
        };

        let mut field_mask = 0_u64;
        let mut initialized: Vec<Option<&HirStructExprField>> = vec![None; strct_def.fields.len()];

        for field in &strct.fields {
            let Some(idx) = strct_def.field_map.get(&field.ident.ident) else {
                self.dctx.add(InferDiag::error(
                    field.ident.span,
                    InferDiagErrorKind::UnrecognizedField {
                        field: field.ident.ident,
                        struct_def: *def_id,
                    },
                ));

                continue;
            };

            // Field is already initialized
            if (field_mask >> idx) & 1 == 1 {
                self.dctx.add(InferDiag::error(
                    field.span(),
                    InferDiagErrorKind::FieldReinitialization {
                        field: field.ident.ident,
                        earlier_span: initialized[*idx].unwrap().span(),
                    },
                ));

                continue;
            }

            field_mask |= 1 << idx;
            initialized[*idx] = Some(&field);

            let field_ty_id = match self.infer_expr(&field.val) {
                Ok(ty_id) => ty_id,
                Err(diag) => {
                    diag.map(|diag| self.dctx.add(diag));
                    continue;
                }
            };

            let Some(t_field_ty) = self.tctx.get_ty_of_hir(strct_def.fields[*idx].ty).cloned()
            else {
                unreachable!()
            };

            if !self.tctx.unify_ty(t_field_ty.id, field_ty_id) {
                self.dctx.add(InferDiag::error(
                    field.val.span,
                    InferDiagErrorKind::TypeMismatch {
                        ann_span: t_field_ty.span,
                        expected: t_field_ty.id,
                        received: field_ty_id,
                    },
                ));
            }
        }

        let missing_mask = !field_mask & ((1 << strct_def.fields.len()) - 1);
        if missing_mask != 0 {
            self.dctx.add(InferDiag::error(
                strct.span,
                InferDiagErrorKind::MissingFields {
                    field_mask: missing_mask,
                    def_id: *def_id,
                },
            ));
        }

        Ok(self.tctx.get_ty_of_hir(def.hir_id).unwrap().id)
    }
}
