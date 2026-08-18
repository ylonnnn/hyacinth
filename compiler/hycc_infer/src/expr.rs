use std::sync::Arc;

use hycc_const::constant::ConstKind;
use hycc_diagnostic::diagnostic::{Diagnostics, FromResultEmitter};
use hycc_hir::{
    HirMutability, HirNode,
    def::{AdtKind, Binding, DefKind, DefSpace},
    expr::{
        HirArrayExpr, HirExpr, HirExprKind, HirFieldAccess, HirFieldAccessFieldKind, HirFnCall,
        HirIfExpr, HirLiteral, HirMethodCall, HirRefExpr, HirStructExpr, HirStructExprField,
        HirTupleExpr,
    },
    generic::HirGenericParamKind,
    item::HirItemKind,
    path::HirIdentArgument,
};
use hycc_resolve::{InstantiateIdent, ResolveExpr};
use hycc_span::Span;
use hycc_ty::{
    ctx::TyId,
    extension::{ExtNominalTargetKind, ExtTargetKind},
    ty::{AccessKind, GenericArg, InferKind, RefMutability, Ty, TyKind},
};
use hycc_util::{bug, ternary};

use crate::{
    diag::{InferDiag, InferDiagErrorKind, InferResult, MemberKind},
    fn_ctx::FnCtx,
    inferer::TyInferer,
};

impl<'i, 'h> ResolveExpr<TyId, InferDiag> for TyInferer<'i, 'h> {
    fn resolve_expr(&mut self, expr: &HirExpr) -> Result<TyId, InferDiag> {
        Ok(self.tctx.expect_hir_ty_id(expr.id))
    }
}

impl<'i, 'h> TyInferer<'i, 'h> {
    pub(crate) fn infer_expr(&mut self, expr: &HirExpr) -> InferResult<TyId> {
        let ty_id = match &expr.kind {
            HirExprKind::Path(path) => self.infer_path(&path),
            HirExprKind::RefExpr(reference) => self.infer_ref_expr(&reference),
            HirExprKind::Literal(_) => self.infer_literal_expr(&expr),
            HirExprKind::Binary(op, left, right) => todo!("infer binary"),
            HirExprKind::Unary(unary) => todo!("infer unary"),
            HirExprKind::Assign(assignee, expr) => todo!("infer assignment"),
            HirExprKind::Block(block) => self.infer_block(&block),
            HirExprKind::Array(array) => self.infer_array_expr(&array),
            HirExprKind::Tuple(tup) => self.infer_tuple_expr(&tup),
            HirExprKind::Struct(_) => self.infer_struct_expr(&expr),
            HirExprKind::AnonFn(_) => self.infer_anon_fn_expr(&expr),
            HirExprKind::FnCall(call) => self.infer_fn_call_expr(&call),
            HirExprKind::FieldAccess(access) => self.infer_field_access_expr(&access),
            HirExprKind::MethodCall(call) => self.infer_method_call_expr(&call),
            HirExprKind::If(ite) => self.infer_if_expr(&ite),
        }?;

        self.tctx.attach_to_hir(expr.id, Ty::new(ty_id, expr.span));

        Ok(ty_id)
    }

    pub(crate) fn infer_literal_expr(&mut self, expr: &HirExpr) -> InferResult<TyId> {
        let HirExprKind::Literal(lit) = &expr.kind else {
            unreachable!()
        };

        let kind = self.const_table.get(lit.const_id());
        Ok(match &kind {
            ConstKind::Int { .. } => self.tctx.make_inferred_ty(expr.span, InferKind::Int),
            ConstKind::Float(_) => self.tctx.make_inferred_ty(expr.span, InferKind::Float),
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
        let el_ty_id = ternary!(
            array.elements.is_empty(),
            self.tctx.make_inferred_ty(array.span, InferKind::Any),
            self.infer_expr(array.elements[0])?
        );

        if !array.elements.is_empty() {
            array.elements[1..].iter().for_each(|el| {
                let Some(curr_el_ty_id) = self.infer_expr(&el).emit(&mut self.dctx) else {
                    return;
                };

                self.check(
                    &Ty::new(el_ty_id, Span::default()),
                    &Ty::new(curr_el_ty_id, el.span),
                )
                .map(|diag| self.dctx.add(diag));

                self.tctx.resolve_ty(curr_el_ty_id);
            });
        }

        Ok(self.tctx.make_array_ty(el_ty_id))
    }

    pub(crate) fn infer_tuple_expr(&mut self, tup: &HirTupleExpr) -> InferResult<TyId> {
        let tys = tup
            .elements
            .iter()
            .filter_map(|el| self.infer_expr(&el).emit(&mut self.dctx))
            .collect::<Arc<_>>();

        Ok(self.tctx.make_tuple_ty(tys))
    }

    pub(crate) fn infer_struct_expr(&mut self, struct_expr: &HirExpr) -> InferResult<TyId> {
        let HirExprKind::Struct(strct) = &struct_expr.kind else {
            unreachable!()
        };

        let err = |name, def_id| {
            Err(InferDiag::error(
                strct.path.span,
                InferDiagErrorKind::InvalidNonStructInstantiation { name, def_id },
            ))
        };

        let Some(TyKind::Adt(def_id, args)) = self
            .tctx
            .get_hir_ty_id(strct.path.id)
            .map(|ty_id| self.tctx.get(ty_id))
        else {
            let def_id = self.definitions.expect_def_id(strct.path.id);
            return err(self.definitions.expect_def(strct.path.id).name, def_id);
        };

        let def_id = *def_id;
        let def = self.definitions.get(def_id);
        let Some(strct_def) = def.kind.get_adt().and_then(|adt| adt.get_struct()) else {
            return err(def.name, def_id);
        };

        let hir_id = def.hir_id;
        let n = strct_def.fields.len();

        let mut field_mask = 0_u64;
        let mut initialized: Vec<Option<&HirStructExprField>> = vec![None; strct_def.fields.len()];

        let field_map = strct_def.field_map.clone();
        let field_tys = strct_def.fields.iter().map(|f| f.ty).collect::<Vec<_>>();

        let TyKind::Adt(_, args) = self.tctx.get(self.tctx.expect_hir_ty_id(strct.path.id)) else {
            unreachable!()
        };

        let args = args.clone();

        for field in &strct.fields {
            let Some(idx) = field_map.get(&field.ident.ident) else {
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

            let raw_field_ty_id = self.tctx.expect_hir_ty_id(field_tys[*idx]);
            let field_ty_id = self.tctx.instantiate(raw_field_ty_id, &[&args]);

            let Some(expr_field_ty_id) = self.infer_expr(&field.val).emit(&mut self.dctx) else {
                continue;
            };

            self.check(
                &Ty::new(field_ty_id, Span::default()),
                &Ty::new(expr_field_ty_id, field.val.span),
            )
            .map(|diag| self.dctx.add(diag));
        }

        let missing_mask = !field_mask & ((1 << n) - 1);
        if missing_mask != 0 {
            self.dctx.add(InferDiag::error(
                strct.span,
                InferDiagErrorKind::MissingFields {
                    field_mask: missing_mask,
                    def_id,
                },
            ));
        }

        Ok(self.tctx.expect_hir_ty_id(strct.path.id))
    }

    pub(crate) fn infer_anon_fn_expr(&mut self, anfn_expr: &HirExpr) -> InferResult<TyId> {
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
            let TyKind::Fn(fn_ty, _) = s.tctx.get(fn_ty_id) else {
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
            let TyKind::Fn(fn_ty, args) = s.tctx.get(fn_ty_id) else {
                unreachable!()
            };

            let resolved_args = args.clone(); // TODO
            let resolved_param_tys = fn_ty
                .params
                .clone()
                .into_iter()
                .map(|param| s.tctx.resolve_ty(*param))
                .collect::<Vec<_>>()
                .into();
            let fn_ty = s
                .tctx
                .make_fn_ty(resolved_args, None, resolved_param_tys, resolved_ret_ty);

            s.tctx
                .attach_to_hir(anfn_expr.id, Ty::new(fn_ty, anfn_expr.span));

            Ok(())
        })?;

        Ok(self.tctx.get_hir_ty(anfn_expr.id).unwrap().id)
    }

    pub(crate) fn infer_fn_call_expr(&mut self, call: &HirFnCall) -> InferResult<TyId> {
        let callee_ty_id = self.infer_expr(&call.callee)?;
        let TyKind::Fn(fn_ty, args) = self.tctx.get(callee_ty_id) else {
            return Err(InferDiag::error(
                call.callee.span,
                InferDiagErrorKind::IllegalInvocation(callee_ty_id),
            ));
        };

        let (a_len, p_len) = (call.arguments.data.len(), fn_ty.params.len());
        if a_len != p_len {
            return Err(InferDiag::error(
                call.arguments.span,
                InferDiagErrorKind::ArgumentArityMismatch(
                    ((p_len as u16) << u8::BITS) | a_len as u16,
                ),
            ));
        }

        let ret_ty = fn_ty.ret_ty;
        let params = fn_ty.params.clone();

        call.arguments
            .data
            .iter()
            .zip(params.iter())
            .for_each(|(arg, param_ty_id)| {
                let Some(arg_ty_id) = self.infer_expr(&arg).emit(&mut self.dctx) else {
                    return;
                };

                self.check(
                    &Ty::new(*param_ty_id, Span::default()),
                    &Ty::new(arg_ty_id, arg.span),
                )
                .map(|diag| self.dctx.add(diag));
            });

        Ok(self.tctx.resolve_ty(ret_ty))
    }

    pub(crate) fn infer_field_access_expr(&mut self, access: &HirFieldAccess) -> InferResult<TyId> {
        let init_lead_ty_id = self.infer_expr(&access.leading)?;
        let mut lead_ty_id = init_lead_ty_id;

        loop {
            let err = || {
                Err(InferDiag::error(
                    access.field.span,
                    InferDiagErrorKind::UnrecognizedField {
                        field: access.field.kind,
                        ty_id: lead_ty_id,
                    },
                ))
            };

            match self.tctx.get(lead_ty_id) {
                TyKind::Tuple(tup) => {
                    return if let HirFieldAccessFieldKind::Index(idx) = &access.field.kind
                        && *idx < tup.len()
                    {
                        Ok(tup[*idx])
                    } else {
                        err()
                    };
                }

                TyKind::Adt(def_id, args) => {
                    let def = self.definitions.get(*def_id);
                    let struct_def = def.kind.expect_adt().expect_struct();

                    let Some(field) = (match &access.field.kind {
                        HirFieldAccessFieldKind::Ident(ident) => struct_def.field_map.get(&ident),
                        _ => None,
                    }) else {
                        return err();
                    };

                    let field_ty_id = self.tctx.expect_hir_ty_id(struct_def.fields[*field].ty);
                    return Ok(self.tctx.instantiate(field_ty_id, &[&args.clone()]));
                }

                TyKind::Ref(ty_id, _) => {
                    lead_ty_id = *ty_id;
                }

                _ => return err(),
            }
        }
    }

    pub fn infer_method_call_expr(&mut self, call: &HirMethodCall) -> InferResult<TyId> {
        let initial_ty_id = self.infer_expr(&call.receiver)?;

        let mut rec_ty_id = initial_ty_id;
        let mut deref_rec_ty_id = self.tctx.deref(rec_ty_id);
        let mut rec_g_args = None;

        let mut candidate = None;
        let mut access = AccessKind::Owned;

        // Find the candidate binding
        loop {
            let target = self.tctx.ext_target_kind_of(rec_ty_id);
            let err = || {
                Err(InferDiag::error(
                    call.callee.span,
                    InferDiagErrorKind::UnrecognizedMethod {
                        method: call.callee.ident.ident,
                        ty_id: rec_ty_id,
                    },
                ))
            };

            if let Some((ext_id, assoc_item)) =
                self.tctx
                    .ext_table
                    .get_assoc_item(target, DefSpace::Value, call.callee.ident.ident)
            {
                let ext = self.tctx.ext_table.get(ext_id);
                let (ext_target_ty_id, ext_hir_id) = (ext.expect_target(), ext.hir_id);

                let HirNode::Item(item) = &self.hir_table.get(ext_hir_id) else {
                    unreachable!()
                };

                let HirItemKind::Extend(extend) = &item.kind else {
                    unreachable!()
                };

                let n = extend
                    .generic_params
                    .as_ref()
                    .map_or(0, |generic_params| generic_params.list.len());

                rec_g_args.replace(
                    (0..n)
                        .map(|_| {
                            GenericArg::Ty(
                                self.tctx.make_inferred_ty(Span::default(), InferKind::Any),
                            )
                        })
                        .collect::<Vec<_>>(),
                );

                let ext_target_ty_id = self
                    .tctx
                    .instantiate(ext_target_ty_id, &[rec_g_args.as_ref().unwrap()]);
                if !self.compatible(ext_target_ty_id, rec_ty_id) {
                    return err();
                }

                candidate.replace(assoc_item);
            };

            if rec_ty_id == deref_rec_ty_id {
                break;
            }

            if let TyKind::Ref(_, mutability) = self.tctx.get(rec_ty_id) {
                access = AccessKind::Ref(*mutability)
            }

            rec_ty_id = deref_rec_ty_id;
            deref_rec_ty_id = self.tctx.deref(rec_ty_id);
        }

        let Some(Binding {
            def_id,
            accessibility,
        }) = candidate
        else {
            Err(InferDiag::error(
                call.callee.span,
                InferDiagErrorKind::UnrecognizedMethod {
                    method: call.callee.ident.ident,
                    ty_id: rec_ty_id,
                },
            ))?
        };

        self.definitions.define_id_hir(call.callee.id, def_id);

        let mut arg_frames = [rec_g_args.unwrap()]
            .into_iter()
            .filter_map(|frame| ternary!(frame.is_empty(), None, Some(frame)))
            .collect::<Vec<_>>();
        let fn_ty_id = self.instantiate(&mut arg_frames, &call.callee)?;

        let def = self.definitions.get(def_id);

        // Illegal invocation error for non-function bindings
        let Some(fn_def) = def.kind.get_fn() else {
            Err(InferDiag::error(
                call.callee.span,
                InferDiagErrorKind::IllegalInvocation(fn_ty_id),
            ))?
        };

        let fn_def_params = fn_def.params.clone();

        self.tctx
            .attach_to_hir(call.callee.id, Ty::new(fn_ty_id, call.callee.span));

        let TyKind::Fn(fn_ty, args) = self.tctx.get(fn_ty_id) else {
            unreachable!()
        };

        let ret_ty = fn_ty.ret_ty;
        let params = fn_ty.params.clone();

        // Accessibility guard
        if !self.petal_ctx.accessible(&def) {
            Err(InferDiag::error(
                call.callee.span,
                InferDiagErrorKind::InaccessibleMember {
                    name: call.callee.ident.ident,
                    kind: MemberKind::AssocFn,
                },
            ))?
        }

        // Associated function attempted to be called through method calling
        let p_len = params.len();
        if p_len < 1 {
            Err(InferDiag::error(
                call.callee.span,
                InferDiagErrorKind::IllegalAssocFnInvocation {
                    name: call.callee.ident.ident,
                    def_id,
                    ty_id: rec_ty_id,
                },
            ))?;
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

            // TODO: check if the receiver type has proto Copy
            if !access.allows(req_access) {
                Err(InferDiag::error(
                    call.receiver.span,
                    InferDiagErrorKind::ReceiverAccessMismatch {
                        access,
                        requested: req_access,
                        call: (
                            call.callee.ident.ident,
                            call.callee.span.merge(call.arguments.span),
                        ),
                        def_id,
                    },
                ))?;
            }

            receiver_ty_id = Some(recv_ty_id);
            break;
        }

        // No method found for the three (3) candidate receiver types.
        // Therefore, the associated function found is not a method.
        let Some(rec_ty_id) = receiver_ty_id else {
            return Err(InferDiag::error(
                call.callee.span,
                InferDiagErrorKind::IllegalAssocFnInvocation {
                    name: call.callee.ident.ident,
                    def_id,
                    ty_id: rec_ty_id,
                },
            ))?;
        };

        // Argument arity guard
        let a_len = call.arguments.data.len();
        if (a_len + 1) != p_len {
            Err(InferDiag::error(
                call.arguments.span,
                InferDiagErrorKind::ArgumentArityMismatch(
                    ((p_len.saturating_sub(1) as u16) << u8::BITS) | a_len as u16,
                ),
            ))?;
        };

        for (i, (arg, param_ty_id)) in std::iter::once(&call.receiver)
            .chain(call.arguments.data.iter())
            .zip(params.iter())
            .enumerate()
        {
            let arg_ty_id = ternary!(
                i == 0,
                rec_ty_id,
                self.infer_expr(&arg)
                    .emit(&mut self.dctx)
                    .unwrap_or(continue)
            );

            self.check(
                &Ty::new(*param_ty_id, self.definitions.get(fn_def_params[i]).span),
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
                Err(InferDiag::error(
                    ite.span,
                    InferDiagErrorKind::MissingElseBranch,
                ))?
            }

            self.dctx.add(diag);
        }

        Ok(cons_ty)
    }
}
