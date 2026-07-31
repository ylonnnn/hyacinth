use hycc_const::constant::ConstKind;
use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirMutability, HirNode,
    def::{AdtKind, DefKind, DefSpace},
    expr::{
        HirArrayExpr, HirExpr, HirExprKind, HirFieldAccess, HirFieldAccessFieldKind, HirFnCall,
        HirIfExpr, HirLiteral, HirMethodCall, HirRefExpr, HirStructExpr, HirStructExprField,
        HirTupleExpr,
    },
    item::HirItemKind,
};
use hycc_span::Span;
use hycc_ty::{
    context::TyId,
    ty::{AccessKind, InferKind, RefMutability, Ty, TyKind},
};
use hycc_util::{bug, ternary};

use crate::{
    diag::{InferDiag, InferDiagErrorKind, MemberKind},
    fn_ctx::FnCtx,
    inferer::{InferResult, TyInferer},
};

impl<'t, 'd, 'c, 'h, 'p> TyInferer<'t, 'd, 'c, 'h, 'p> {
    pub(crate) fn infer_expr(&mut self, expr: &HirExpr) -> InferResult<TyId> {
        let ty_id = match &expr.kind {
            HirExprKind::Path(path) => self.infer_path(&path),
            HirExprKind::RefExpr(reference) => self.infer_ref_expr(&reference),
            HirExprKind::Literal(lit) => self.infer_literal(&lit),
            HirExprKind::Binary(op, left, right) => todo!("infer binary"),
            HirExprKind::Unary(unary) => todo!("infer unary"),
            HirExprKind::Assign(assignee, expr) => todo!("infer assignment"),
            HirExprKind::Block(block) => self.infer_block(&block),
            HirExprKind::Array(array) => self.infer_array_expr(&array),
            HirExprKind::Tuple(tup) => self.infer_tuple_expr(&tup),
            HirExprKind::Struct(strct) => self.infer_struct_expr(&strct),
            HirExprKind::AnonFn(_) => self.infer_anon_fn(&expr),
            HirExprKind::FnCall(call) => self.infer_fn_call(&call),
            HirExprKind::FieldAccess(access) => self.infer_field_access(&access),
            HirExprKind::MethodCall(call) => self.infer_method_call(&call),
            HirExprKind::If(ite) => self.infer_if_expr(&ite),
        }?;

        self.tctx.attach_to_hir(expr.id, Ty::new(ty_id, expr.span));

        Ok(ty_id)
    }

    pub(crate) fn infer_literal(&mut self, lit: &HirLiteral) -> InferResult<TyId> {
        let kind = self.const_table.get(lit.const_id());
        Ok(match &kind {
            ConstKind::Int { .. } => self.tctx.make_inferred_ty(InferKind::Int),
            ConstKind::Float(_) => self.tctx.make_inferred_ty(InferKind::Float),
            ConstKind::Bool(_) => self.tctx.make_bool_ty(),
            ConstKind::Char(_) => self.tctx.make_char_ty(),
            ConstKind::String(_) => self.tctx.make_string_ty(),
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
        // TODO: improve. use initial state verification for micro-optimization
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

            self.check(
                &Ty::new(el_ty_id, Span::default()),
                &Ty::new(curr_el_ty_id, expr.span),
            )
            .map(|diag| self.dctx.add(diag));

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
        let Some(def_id) = self.definitions.get_def_id(strct.path.id) else {
            unreachable!()
        };

        let def = self.definitions.get(def_id);
        let DefKind::Adt(AdtKind::Struct(strct_def)) = &def.kind else {
            return Err(Some(InferDiag::error(
                strct.path.span,
                InferDiagErrorKind::InvalidNonStructInstantiation {
                    name: def.name,
                    def_id,
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
                        struct_def: def_id,
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

            let Some(t_field_ty) = self.tctx.get_hir_ty(strct_def.fields[*idx].ty).cloned() else {
                unreachable!()
            };

            self.check(&t_field_ty, &Ty::new(field_ty_id, field.val.span))
                .map(|diag| self.dctx.add(diag));
        }

        let missing_mask = !field_mask & ((1 << strct_def.fields.len()) - 1);
        if missing_mask != 0 {
            self.dctx.add(InferDiag::error(
                strct.span,
                InferDiagErrorKind::MissingFields {
                    field_mask: missing_mask,
                    def_id,
                },
            ));
        }

        Ok(self.tctx.get_hir_ty(def.hir_id).unwrap().id)
    }

    pub(crate) fn infer_anon_fn(&mut self, anfn_expr: &HirExpr) -> InferResult<TyId> {
        let HirExprKind::AnonFn(anfn) = &anfn_expr.kind else {
            unreachable!()
        };

        let Some(fn_ty) = self.tctx.get_hir_ty(anfn_expr.id).cloned() else {
            bug!(
                "anon fn hir {:?} does not have an attached ty",
                anfn_expr.id
            )
        };

        let fn_ty_id = fn_ty.id;

        self.use_fn_ctx(FnCtx::new(fn_ty, anfn.body.id), |s| -> InferResult {
            let TyKind::Fn(fn_ty) = s.tctx.get(fn_ty_id) else {
                return Ok(());
            };

            let ret_ty = Ty::new(
                fn_ty.ret_ty,
                anfn.ret_ty.map(|ty| ty.span).unwrap_or(Span::default()),
            );
            s.tctx.attach_to_hir(anfn.body.id, ret_ty.clone());

            let block_ty_id = s.infer_block(&anfn.body)?;

            s.check(&ret_ty, &Ty::new(block_ty_id, anfn.body.span))
                .map(|diag| s.dctx.add(diag));

            let resolved_ret_ty = s.tctx.resolve_ty(ret_ty.id);
            let TyKind::Fn(fn_ty) = s.tctx.get(fn_ty_id) else {
                unreachable!()
            };

            let resolved_param_tys = fn_ty
                .params
                .clone()
                .into_iter()
                .map(|param| s.tctx.resolve_ty(*param))
                .collect::<Vec<_>>()
                .into();
            let fn_ty = s.tctx.make_fn_ty(resolved_param_tys, resolved_ret_ty);

            s.tctx
                .attach_to_hir(anfn_expr.id, Ty::new(fn_ty, anfn_expr.span));

            Ok(())
        })?;

        Ok(self.tctx.get_hir_ty(anfn_expr.id).unwrap().id)
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
                InferDiagErrorKind::ArgumentArityMismatch {
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

            self.check(
                &Ty::new(*param_ty_id, Span::default()),
                &Ty::new(arg_ty_id, arg.span),
            )
            .map(|diag| self.dctx.add(diag));
        }

        Ok(self.tctx.resolve_ty(ret_ty))
    }

    pub(crate) fn infer_field_access(&mut self, access: &HirFieldAccess) -> InferResult<TyId> {
        let init_lead_ty_id = self.infer_expr(&access.leading)?;
        let mut lead_ty_id = init_lead_ty_id;

        loop {
            let lead_ty_kind = self.tctx.get(lead_ty_id);
            let err = || {
                Err(Some(InferDiag::error(
                    access.field.span,
                    InferDiagErrorKind::UnrecognizedField {
                        field: access.field.kind,
                        ty_id: lead_ty_id,
                    },
                )))
            };

            match lead_ty_kind {
                TyKind::Tuple(tup) => {
                    return if let HirFieldAccessFieldKind::Index(idx) = &access.field.kind
                        && *idx < tup.len()
                    {
                        Ok(tup[*idx])
                    } else {
                        err()
                    };
                }

                TyKind::Adt(def_id) => {
                    let def = self.definitions.get(*def_id);
                    let DefKind::Adt(AdtKind::Struct(struct_def)) = &def.kind else {
                        unreachable!()
                    };

                    let Some(field) = (match &access.field.kind {
                        HirFieldAccessFieldKind::Ident(ident) => struct_def.field_map.get(&ident),
                        _ => None,
                    }) else {
                        return err();
                    };

                    let field = &struct_def.fields[*field];
                    let Some(field_ty) = self.tctx.get_hir_ty(field.ty) else {
                        bug!("hir {:?} does not have an attached ty_id", field.ty)
                    };

                    return Ok(field_ty.id);
                }

                TyKind::Ref(ty_id, _) => {
                    lead_ty_id = *ty_id;
                }

                _ => return err(),
            }
        }
    }

    pub fn infer_method_call(&mut self, call: &HirMethodCall) -> InferResult<TyId> {
        let initial_ty_id = self.infer_expr(&call.receiver)?;

        let mut rec_ty_id = initial_ty_id;
        let mut deref_rec_ty_id = self.tctx.deref(rec_ty_id);

        let mut candidate = None;
        let mut access = AccessKind::Owned;

        // Find the candidate binding
        loop {
            if let Some(exts) = self
                .tctx
                .get_ty_def_id(rec_ty_id)
                .and_then(|def_id| self.tctx.ext_table.get_def_native_exts(def_id))
            {
                if let Some(binding) = exts.iter().find_map(|ext_id| {
                    self.tctx
                        .ext_table
                        .get(*ext_id)
                        .get(DefSpace::Value, call.callee.ident.ident)
                }) {
                    candidate.replace(binding.clone());
                    break;
                }
            }

            if rec_ty_id == deref_rec_ty_id {
                break;
            }

            if let TyKind::Ref(_, mutability) = self.tctx.get(rec_ty_id) {
                access = AccessKind::Ref(*mutability)
            }

            rec_ty_id = deref_rec_ty_id;
            deref_rec_ty_id = self.tctx.deref(rec_ty_id);
        }

        let Some(binding) = candidate else {
            Err(Some(InferDiag::error(
                call.callee.span,
                InferDiagErrorKind::UnrecognizedMethod {
                    method: call.callee.ident.ident,
                    ty_id: rec_ty_id,
                },
            )))?
        };

        let def = self.definitions.get(binding.def_id);
        let fn_ty = self.tctx.get_ty_of_def(binding.def_id).unwrap();

        // Illegal invocation error for non-function bindings
        let DefKind::Fn(fn_def) = &def.kind else {
            Err(Some(InferDiag::error(
                call.callee.span,
                InferDiagErrorKind::IllegalInvocation(fn_ty.id),
            )))?
        };

        let TyKind::Fn(fn_ty) = self.tctx.get(fn_ty.id) else {
            unreachable!()
        };

        let ret_ty = fn_ty.ret_ty;
        let params = fn_ty.params.clone();

        // Accessibility guard
        if !self.petal_ctx.accessible(&def) {
            Err(Some(InferDiag::error(
                call.callee.span,
                InferDiagErrorKind::InaccessibleMember {
                    name: call.callee.ident.ident,
                    kind: MemberKind::AssocFn,
                },
            )))?
        }

        // Associated function attempted to be called through method calling
        let p_len = params.len();
        if p_len < 1 {
            Err(Some(InferDiag::error(
                call.callee.span,
                InferDiagErrorKind::InvalidAssocFnInvocation {
                    name: call.callee.ident.ident,
                    def_id: binding.def_id,
                    ty_id: rec_ty_id,
                },
            )))?;
        }

        // Check the valid type candidate for the method
        let mut receiver_ty_id = None;
        for i in 0..3 {
            let req_access = match i {
                0 => AccessKind::Owned,
                1 => AccessKind::Ref(RefMutability::Immutable),
                _ => AccessKind::Ref(RefMutability::Mutable),
            };

            let recv_ty_id = ternary!(
                i == 0,
                rec_ty_id,
                self.tctx.make_ref_ty(
                    rec_ty_id,
                    ternary!(i == 1, RefMutability::Immutable, RefMutability::Mutable)
                )
            );

            if !self.compatible(params[0], recv_ty_id) {
                continue;
            }

            if !access.allows(req_access) {
                Err(Some(InferDiag::error(
                    call.receiver.span,
                    InferDiagErrorKind::ReceiverAccessMismatch {
                        access,
                        requested: req_access,
                        call: (
                            call.callee.ident.ident,
                            call.callee.span.merge(&call.arguments.span),
                        ),
                        def_id: binding.def_id,
                    },
                )))?;
            }

            receiver_ty_id = Some(recv_ty_id);
            break;
        }

        // No method found for the three (3) candidate receiver types.
        // Therefore, the associated function found is not a method.
        let Some(rec_ty_id) = receiver_ty_id else {
            return Err(Some(InferDiag::error(
                call.callee.span,
                InferDiagErrorKind::InvalidAssocFnInvocation {
                    name: call.callee.ident.ident,
                    def_id: binding.def_id,
                    ty_id: rec_ty_id,
                },
            )))?;
        };

        // Argument arity guard
        let a_len = call.arguments.data.len();
        if (a_len + 1) != p_len {
            Err(Some(InferDiag::error(
                call.arguments.span,
                InferDiagErrorKind::ArgumentArityMismatch {
                    expected: p_len.saturating_sub(1) as u8,
                    received: a_len as u8,
                },
            )))?;
        }

        for (i, (arg, param_ty_id)) in std::iter::once(&call.receiver)
            .chain(call.arguments.data.iter())
            .zip(params.iter())
            .enumerate()
        {
            let arg_ty_id = ternary!(
                i == 0,
                rec_ty_id,
                match self.infer_expr(&arg) {
                    Ok(ty_id) => ty_id,
                    Err(diag) => {
                        diag.map(|diag| self.dctx.add(diag));
                        continue;
                    }
                }
            );

            self.check(
                &Ty::new(*param_ty_id, self.definitions.get(fn_def.params[i]).span),
                &Ty::new(arg_ty_id, arg.span),
            )
            .map(|diag| self.dctx.add(diag));
        }

        Ok(self.tctx.resolve_ty(ret_ty))
    }

    pub fn infer_if_expr(&mut self, ite: &HirIfExpr) -> InferResult<TyId> {
        let bool_ty = self.tctx.make_bool_ty();
        let cond_ty = self.infer_expr(&ite.cond)?;

        self.check(
            &Ty::new(bool_ty, Span::default()),
            &Ty::new(cond_ty, ite.cond.span),
        )
        .map(|diag| self.dctx.add(diag));

        let cons_ty = self.infer_block(&ite.consequent)?;
        let (alt_ty, alt_span) = if let Some(alt) = &ite.alternate {
            (self.infer_block(&alt)?, alt.span)
        } else {
            (self.tctx.make_unit_ty(), ite.span)
        };

        if let Some(diag) = self.check(
            &Ty::new(cons_ty, ite.consequent.span),
            &Ty::new(alt_ty, alt_span),
        ) {
            if ite.alternate.is_none() {
                Err(Some(InferDiag::error(
                    ite.span,
                    InferDiagErrorKind::MissingElseBranch,
                )))?
            }

            self.dctx.add(diag);
        }

        Ok(cons_ty)
    }
}
