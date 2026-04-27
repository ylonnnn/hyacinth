use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirMutability,
    def::DefKind,
    expr::{
        HirArrayExpr, HirExpr, HirExprKind, HirFieldAccess, HirFieldAccessFieldKind, HirFnCall,
        HirLiteral, HirRefExpr, HirStructExpr, HirStructExprField, HirTupleExpr,
    },
};
use hycc_span::Span;
use hycc_ty::{
    context::TyId,
    ty::{InferKind, RefMutability, TyKind},
};
use hycc_util::{bug, ternary};

use crate::{
    diag::{InferDiag, InferDiagErrorKind},
    inferer::{InferResult, TyInferer},
};

impl<'t, 'd, 'r> TyInferer<'t, 'd, 'r> {
    pub(crate) fn infer_expr(&mut self, expr: &HirExpr) -> InferResult<TyId> {
        match &expr.kind {
            HirExprKind::Path(path) => self.infer_path(&path),
            HirExprKind::RefExpr(reference) => self.infer_ref_expr(&reference),
            HirExprKind::Literal(lit) => self.infer_literal(&lit),
            HirExprKind::Binary(op, left, right) => todo!("infer binary"),
            HirExprKind::Unary(unary) => todo!("infer unary"),
            HirExprKind::Assign(assignee, expr) => todo!("infer assignment"),
            HirExprKind::Block(block) => todo!("infer block"),
            HirExprKind::Array(array) => self.infer_array_expr(&array),
            HirExprKind::Tuple(tup) => self.infer_tuple_expr(&tup),
            HirExprKind::Struct(strct) => self.infer_struct_expr(&strct),
            HirExprKind::FnCall(call) => self.infer_fn_call(&call),
            HirExprKind::FieldAccess(access) => self.infer_field_access(&access),
            HirExprKind::MethodCall(call) => todo!("infer method call"),
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

    pub(crate) fn infer_tuple_expr(&mut self, tup: &HirTupleExpr) -> InferResult<TyId> {
        let mut tys = Vec::new();

        for el in &tup.elements {
            match self.infer_expr(&el) {
                Ok(ty_id) => tys.push(ty_id),
                Err(diag) => {
                    if let Some(diag) = diag {
                        self.dctx.add(diag);
                    }

                    continue;
                }
            }
        }

        Ok(self.tctx.make_tuple_ty(tys.into()))
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
                    InferDiagErrorKind::UnrecognizedFieldInitialization {
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

    pub(crate) fn infer_fn_call(&mut self, call: &HirFnCall) -> InferResult<TyId> {
        let callee_ty_id = self.infer_expr(&call.callee)?;
        let TyKind::Fn(fn_ty) = self.tctx.get(callee_ty_id) else {
            return Err(Some(InferDiag::error(
                call.callee.span,
                InferDiagErrorKind::IllegalInvocation(callee_ty_id),
            )));
        };

        let (a_len, p_len) = (call.arguments.data.len(), fn_ty.params.len());
        if a_len != p_len {
            return Err(Some(InferDiag::error(
                call.arguments.span,
                InferDiagErrorKind::InvalidArgumentCount {
                    expected: p_len as u8,
                    received: a_len as u8,
                },
            )));
        }

        let ret_ty = fn_ty.ret_ty;
        let params = fn_ty.params.clone();

        for (arg, param_ty_id) in call.arguments.data.iter().zip(params.iter()) {
            let arg_ty_id = match self.infer_expr(&arg) {
                Ok(ty_id) => ty_id,
                Err(diag) => {
                    if let Some(diag) = diag {
                        self.dctx.add(diag);
                    }

                    continue;
                }
            };

            if !self.tctx.unify_ty(*param_ty_id, arg_ty_id) {
                self.dctx.add(InferDiag::error(
                    arg.span,
                    InferDiagErrorKind::TypeMismatch {
                        ann_span: Span::default(),
                        expected: *param_ty_id,
                        received: arg_ty_id,
                    },
                ));
            }
        }

        Ok(ret_ty)
    }

    pub(crate) fn infer_field_access(&mut self, access: &HirFieldAccess) -> InferResult<TyId> {
        let init_lead_ty_id = self.infer_expr(&access.leading)?;
        let mut lead_ty_id = init_lead_ty_id;

        let err = Err(Some(InferDiag::error(
            access.field.span,
            InferDiagErrorKind::UnrecognizedField {
                field: access.field.kind,
                ty_id: init_lead_ty_id,
            },
        )));

        loop {
            let lead_ty_kind = self.tctx.get(lead_ty_id);
            match &lead_ty_kind {
                TyKind::Tuple(tup) => {
                    return if let HirFieldAccessFieldKind::Index(idx) = &access.field.kind
                        && *idx < tup.len()
                    {
                        Ok(tup[*idx])
                    } else {
                        err
                    };
                }

                TyKind::Adt(def_id) => {
                    let def = self.definitions.get(*def_id);
                    let DefKind::Struct(struct_def) = &def.kind else {
                        unreachable!()
                    };

                    let Some(field) = (match &access.field.kind {
                        HirFieldAccessFieldKind::Ident(ident) => struct_def.field_map.get(&ident),
                        _ => None,
                    }) else {
                        return err;
                    };

                    let field = &struct_def.fields[*field];
                    let Some(field_ty) = self.tctx.get_ty_of_hir(field.ty) else {
                        bug!("hir {:?} does not have an attached ty_id", field.ty)
                    };

                    return Ok(field_ty.id);
                }

                TyKind::Ref(ty_id, _) => {
                    lead_ty_id = *ty_id;
                }

                _ => return err,
            }
        }
    }
}
