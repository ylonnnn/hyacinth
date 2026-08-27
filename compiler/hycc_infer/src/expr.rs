use std::sync::Arc;

use hycc_const::constant::ConstKind;
use hycc_diagnostic::diagnostic::{Diagnostics, FromResultEmitter};
use hycc_hir::{
    HirMutability, HirNode,
    def::{AdtKind, Binding, DefKind, DefSpace},
    expr::{
        HirArrayExpr, HirCastExpr, HirExpr, HirExprKind, HirFieldAccess, HirFieldAccessFieldKind,
        HirFnCall, HirIfExpr, HirLiteral, HirMethodCall, HirRefExpr, HirStructExpr,
        HirStructExprField, HirTupleExpr,
    },
    generic::HirGenericParamKind,
    item::HirItemKind,
    path::HirIdentArgument,
};
use hycc_resolve::{InstantiateIdent, ResolveExpr, ResolvePath, diag::SymbolKind};
use hycc_span::Span;
use hycc_ty::{
    ctx::{TyId, TyResState},
    extension::{ExtNominalTargetKind, ExtTargetKind},
    ty::{AccessKind, GenericArg, InferKind, RefMutability, Ty, TyKind},
};
use hycc_util::{bug, ternary};

use crate::{
    diag::{InferDiag, InferDiagErrorKind, InferResult},
    fn_ctx::FnCtx,
    inferer::TyInferer,
};

impl<'i, 'h> ResolveExpr<TyId, InferDiag> for TyInferer<'i, 'h> {
    fn resolve_expr(&mut self, expr: &HirExpr) -> Result<TyId, InferDiag> {
        Ok(self.tctx.expect_hir_ty_id(expr.id))
    }
}

impl<'i, 'h> TyInferer<'i, 'h> {
    pub(crate) fn check_expr(&mut self, expr: &HirExpr) -> InferResult {
        match &expr.kind {
            HirExprKind::Path(path) => Ok(()),
            HirExprKind::RefExpr(reference) => self.check_ref_expr(&reference),
            HirExprKind::Literal(_) => Ok(()),
            HirExprKind::Binary(op, left, right) => todo!("check binary"),
            HirExprKind::Unary(unary) => todo!("check unary"),
            HirExprKind::Cast(cast) => self.check_cast_expr(&cast),
            HirExprKind::Assign(assignee, expr) => todo!("check assignment"),
            HirExprKind::Block(block) => self.check_block(&block),
            HirExprKind::Array(array) => self.check_array_expr(&array),
            HirExprKind::Tuple(tup) => self.check_tuple_expr(&tup),
            HirExprKind::Struct(_) => self.check_struct_expr(&expr),
            HirExprKind::AnonFn(_) => self.check_anon_fn_expr(&expr),
            HirExprKind::FnCall(call) => self.check_fn_call_expr(&call),
            HirExprKind::FieldAccess(access) => self.check_field_access_expr(&access),
            HirExprKind::MethodCall(call) => self.check_method_call_expr(&call),
            HirExprKind::If(ite) => self.check_if_expr(&ite),
        }
    }

    pub(crate) fn infer_expr(
        &mut self,
        expr: &HirExpr,
        expected_ty: Option<Ty>,
    ) -> InferResult<TyId> {
        let ty_id = match &expr.kind {
            HirExprKind::Path(path) => self.resolve_path(&path),
            HirExprKind::RefExpr(reference) => self.infer_ref_expr(&reference),
            HirExprKind::Literal(_) => self.infer_literal_expr(&expr),
            HirExprKind::Binary(op, left, right) => todo!("infer binary"),
            HirExprKind::Unary(unary) => todo!("infer unary"),
            HirExprKind::Cast(cast) => self.infer_cast_expr(&cast),
            HirExprKind::Assign(assignee, expr) => todo!("infer assignment"),
            HirExprKind::Block(block) => self.infer_block(&block),
            HirExprKind::Array(array) => self.infer_array_expr(&array),
            HirExprKind::Tuple(tup) => self.infer_tuple_expr(&tup),
            HirExprKind::Struct(_) => self.infer_struct_expr(&expr, expected_ty),
            HirExprKind::AnonFn(_) => self.infer_anon_fn_expr(&expr, expected_ty),
            HirExprKind::FnCall(call) => self.infer_fn_call_expr(&call, expected_ty),
            HirExprKind::FieldAccess(access) => self.infer_field_access_expr(&access),
            HirExprKind::MethodCall(call) => self.infer_method_call_expr(&call),
            HirExprKind::If(ite) => self.infer_if_expr(&ite),
        }?;

        expected_ty.map(|ty| {
            self.check(&ty, &Ty::new(ty_id, expr.span))
                .emit(&mut self.dctx)
        });

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

    pub(crate) fn check_ref_expr(&mut self, reference: &HirRefExpr) -> InferResult {
        self.check_expr(&reference.expr)
    }

    pub(crate) fn infer_ref_expr(&mut self, reference: &HirRefExpr) -> InferResult<TyId> {
        let inner_ty = self.infer_expr(&reference.expr, None)?;
        let mutability = ternary!(
            reference.mutability == HirMutability::Mutable,
            RefMutability::Mutable,
            RefMutability::Immutable
        );

        Ok(self.tctx.make_ref_ty(inner_ty, mutability))
    }

    pub(crate) fn check_cast_expr(&mut self, cast: &HirCastExpr) -> InferResult {
        self.check_expr(&cast.expr)
    }

    pub(crate) fn infer_cast_expr(&mut self, cast: &HirCastExpr) -> InferResult<TyId> {
        let ty_id = self.infer_expr(&cast.expr, None)?;
        let cast_ty_id = self.tctx.expect_hir_ty_id(cast.ty.id);

        self.cast(
            &Ty::new(ty_id, cast.expr.span),
            &Ty::new(cast_ty_id, cast.ty.span),
        )
        .and_then(|_| Ok(cast_ty_id))
    }

    pub(crate) fn check_array_expr(&mut self, array: &HirArrayExpr) -> InferResult {
        array
            .elements
            .iter()
            .for_each(|el| self.check_expr(&el).emit_discard(&mut self.dctx));

        Ok(())
    }

    pub(crate) fn infer_array_expr(&mut self, array: &HirArrayExpr) -> InferResult<TyId> {
        let el_ty_id = ternary!(
            array.elements.is_empty(),
            self.tctx.make_inferred_ty(array.span, InferKind::Any),
            self.infer_expr(array.elements[0], None)?
        );

        if !array.elements.is_empty() {
            array.elements[1..].iter().for_each(|el| {
                self.infer_expr(&el, Some(Ty::new(el_ty_id, array.elements[0].span)))
                    .emit_discard(&mut self.dctx)
            });
        }

        Ok(self.tctx.make_array_ty(el_ty_id))
    }

    pub(crate) fn check_tuple_expr(&mut self, tup: &HirTupleExpr) -> InferResult {
        tup.elements
            .iter()
            .for_each(|el| self.check_expr(&el).emit_discard(&mut self.dctx));

        Ok(())
    }

    pub(crate) fn infer_tuple_expr(&mut self, tup: &HirTupleExpr) -> InferResult<TyId> {
        let tys = tup
            .elements
            .iter()
            .filter_map(|el| self.infer_expr(&el, None).emit(&mut self.dctx))
            .collect::<Arc<_>>();

        Ok(self.tctx.make_tuple_ty(tys))
    }

    pub(crate) fn check_struct_expr(&mut self, expr: &HirExpr) -> InferResult {
        let HirExprKind::Struct(strct) = &expr.kind else {
            unreachable!()
        };

        strct
            .fields
            .iter()
            .for_each(|field| self.check_expr(&field.val).emit_discard(&mut self.dctx));

        Ok(())
    }

    pub(crate) fn infer_struct_expr(
        &mut self,
        expr: &HirExpr,
        expected_ty: Option<Ty>,
    ) -> InferResult<TyId> {
        let HirExprKind::Struct(strct) = &expr.kind else {
            unreachable!()
        };

        let err_ty = self.tctx.make_error_ty();
        let err = |name, def_id| {
            Err(InferDiag::error(
                strct.path.span,
                InferDiagErrorKind::InvalidNonStructInstantiation { name, def_id },
            ))
        };

        let Some(ty_id) = self.tctx.get_hir_ty_id(strct.path.id) else {
            let Some(def_id) = self.definitions.get_def_id(strct.path.id) else {
                return Ok(err_ty);
            };

            return err(self.definitions.get(def_id).name, def_id);
        };

        if self.tctx.is_error_ty(ty_id) {
            return Ok(err_ty);
        }

        // TODO: somehow use expected_ty
        let TyKind::Adt(def_id, args) = self.tctx.get(ty_id) else {
            let Some(def_id) = self.definitions.get_def_id(strct.path.id) else {
                return Ok(err_ty);
            };

            return err(self.definitions.get(def_id).name, def_id);
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

            let field_ty_hir_id = field_tys[*idx];
            let HirNode::Ty(field_ty) = self.hir_table.get(field_ty_hir_id) else {
                unreachable!()
            };

            let (raw_field_ty_id, field_ty_span) =
                (self.tctx.expect_hir_ty_id(field_ty_hir_id), field_ty.span);
            let field_ty_id = self.tctx.instantiate(raw_field_ty_id, &[&args]);

            let Some(expr_field_ty_id) = self
                .infer_expr(&field.val, Some(Ty::new(field_ty_id, field_ty_span)))
                .emit(&mut self.dctx)
            else {
                continue;
            };

            self.check(
                &Ty::new(field_ty_id, field_ty_span),
                &Ty::new(expr_field_ty_id, field.val.span),
            )
            .emit(&mut self.dctx);
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

    pub(crate) fn check_anon_fn_expr(&mut self, expr: &HirExpr) -> InferResult {
        let HirExprKind::AnonFn(anfn) = &expr.kind else {
            unreachable!()
        };

        let fn_ty = self.tctx.expect_hir_ty(expr.id).clone();
        let fn_ty_id = fn_ty.id;

        self.check_block(&anfn.body)

        // match self.tctx.expect_hir_res_state(expr.id) {
        //     TyResState::Resolved(_) => {
        //         // Recursively check each item of the body without
        //         // performing unnecessary expression type inference
        //         self.check_block(&anfn.body)
        //     }

        //     _ => {
        //         self.infer_anon_fn_expr(&expr).emit(&mut self.dctx);

        //         let TyKind::Fn(fn_ty, _) = self.tctx.get(fn_ty.id) else {
        //             unreachable!()
        //         };

        //         Ok(())
        //     }
        // }
    }

    pub(crate) fn infer_anon_fn_expr(
        &mut self,
        expr: &HirExpr,
        expected_ty: Option<Ty>,
    ) -> InferResult<TyId> {
        let HirExprKind::AnonFn(anfn) = &expr.kind else {
            unreachable!()
        };

        let fn_ty = self.tctx.expect_hir_ty(expr.id).clone();
        let fn_ty_id = fn_ty.id;

        self.use_fn_ctx(FnCtx::new(fn_ty, anfn.body.id), |s| -> InferResult {
            let TyKind::Fn(fn_ty, _) = s.tctx.get(fn_ty_id) else {
                unreachable!()
            };

            let (params, ret_ty) = (fn_ty.params.clone(), fn_ty.ret_ty);

            let resolved_param_tys = params
                .clone()
                .into_iter()
                .map(|param| s.tctx.resolve_ty(*param))
                .collect::<Arc<_>>();

            let ret_ty = Ty::new(
                ret_ty,
                anfn.ret_ty.map(|ty| ty.span).unwrap_or(Span::default()),
            );
            let resolved_ret_ty = s.tctx.resolve_ty(ret_ty.id);

            let fn_ty = s
                .tctx
                .make_fn_ty(Arc::new([]), None, resolved_param_tys, resolved_ret_ty);
            expected_ty.map(|ty| s.tctx.unify_ty(ty.id, fn_ty));

            s.tctx.attach_to_hir(expr.id, Ty::new(fn_ty, expr.span));

            s.tctx.attach_to_hir(anfn.body.id, ret_ty.clone());

            let block_ty_id = s.infer_block(&anfn.body)?;
            s.check(&ret_ty, &Ty::new(block_ty_id, anfn.body.span))
                .emit(&mut s.dctx);

            s.analyze_unresolved();

            Ok(())
        })?;

        Ok(self.tctx.get_hir_ty(expr.id).unwrap().id)
    }

    pub(crate) fn check_fn_call_expr(&mut self, call: &HirFnCall) -> InferResult {
        self.check_expr(&call.callee).emit(&mut self.dctx);
        call.arguments
            .data
            .iter()
            .for_each(|argument| self.check_expr(&argument).emit_discard(&mut self.dctx));

        Ok(())
    }

    pub(crate) fn infer_fn_call_expr(
        &mut self,
        call: &HirFnCall,
        expected_ty: Option<Ty>,
    ) -> InferResult<TyId> {
        let callee_ty_id = self.infer_expr(&call.callee, None)?;
        let TyKind::Fn(fn_ty, args) = self.tctx.get(callee_ty_id) else {
            return ternary!(
                self.tctx.is_error_ty(callee_ty_id),
                Ok(callee_ty_id),
                Err(InferDiag::error(
                    call.callee.span,
                    InferDiagErrorKind::IllegalInvocation(callee_ty_id),
                ))
            );
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

        let def_params = fn_ty
            .def_id
            .map(|def_id| self.definitions.get(def_id).kind.expect_fn().params.clone());

        call.arguments
            .data
            .iter()
            .zip(params.iter())
            .enumerate()
            .for_each(|(i, (arg, param_ty_id))| {
                let def_param = def_params
                    .as_ref()
                    .and_then(|params| params.get(i).cloned());

                let param_def = def_param.map(|def_id| self.definitions.get(def_id));
                self.infer_expr(
                    &arg,
                    Some(Ty::new(
                        *param_ty_id,
                        param_def.map_or_else(|| Span::default(), |def| def.span),
                    )),
                )
                .emit(&mut self.dctx);
            });

        Ok(self.tctx.resolve_ty(ret_ty))
    }

    pub(crate) fn check_field_access_expr(&mut self, access: &HirFieldAccess) -> InferResult {
        self.check_expr(&access.leading).emit(&mut self.dctx);
        Ok(())
    }

    pub(crate) fn infer_field_access_expr(&mut self, access: &HirFieldAccess) -> InferResult<TyId> {
        let init_lead_ty_id = self.infer_expr(&access.leading, None)?;
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

    pub(crate) fn check_method_call_expr(&mut self, call: &HirMethodCall) -> InferResult {
        self.check_expr(&call.receiver).emit(&mut self.dctx);
        // TODO: check expression identifier arguments
        // call.callee

        call.arguments
            .data
            .iter()
            .for_each(|argument| self.check_expr(&argument).emit_discard(&mut self.dctx));

        Ok(())
    }

    pub(crate) fn infer_method_call_expr(&mut self, call: &HirMethodCall) -> InferResult<TyId> {
        let initial_ty_id = self.infer_expr(&call.receiver, None)?;
        if self.tctx.is_error_ty(initial_ty_id) {
            return Ok(initial_ty_id);
        }

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

            let assoc_items = self
                .tctx
                .ext_table
                .get_assoc_items(target, DefSpace::Value, call.callee.ident.ident)
                .into_iter()
                .filter(|(ext_id, assoc_item)| {
                    let ext = self.tctx.ext_table.get(*ext_id);
                    let (ext_target_ty_id, ext_hir_id) = (ext.expect_target(), ext.hir_id);

                    let HirNode::Item(item) = &self.hir_table.get(ext_hir_id) else {
                        unreachable!()
                    };

                    let extend = item.expect_extend();
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
                    self.compatible(ext_target_ty_id, rec_ty_id)
                })
                .collect::<Vec<_>>();

            if assoc_items.is_empty() {
                return err();
            }

            if assoc_items.len() > 1 {
                return Err(InferDiag::error(
                    call.callee.span,
                    InferDiagErrorKind::MultipleAssocItemsMatched {
                        name: call.callee.ident.ident,
                        matches: assoc_items,
                    },
                ));
            }

            candidate.replace(assoc_items[0].1.clone());

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

        // Accessibility guard
        let def = self.definitions.get(def_id);
        if !self.petal_ctx.accessible(&def) {
            Err(InferDiag::error(
                call.callee.span,
                InferDiagErrorKind::Inaccessible {
                    name: call.callee.ident.ident,
                    kind: Some(SymbolKind::AssocItem),
                },
            ))?
        }

        let mut arg_frames = [rec_g_args.unwrap()]
            .into_iter()
            .filter_map(|frame| ternary!(frame.is_empty(), None, Some(frame)))
            .collect::<Vec<_>>();
        let fn_ty_id = self.instantiate(&mut arg_frames, &call.callee)?;
        // self.dctx.add(InferDiag::error(
        //     call.callee.span,
        //     InferDiagErrorKind::IllegalInvocation(fn_ty_id),
        // ));

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

            // TODO: check if the receiver type has intf Copy
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
            if i > 0 {
                self.infer_expr(
                    &arg,
                    Some(Ty::new(
                        *param_ty_id,
                        self.definitions.get(fn_def_params[i]).span,
                    )),
                )
                .emit(&mut self.dctx);
            }
        }

        Ok(self.tctx.resolve_ty(ret_ty))
    }

    pub(crate) fn check_if_expr(&mut self, ite: &HirIfExpr) -> InferResult {
        self.check_expr(&ite.cond).emit(&mut self.dctx);
        self.check_block(&ite.consequent).emit(&mut self.dctx);
        ite.alternate
            .as_ref()
            .map(|alt| self.check_block(&alt).emit_discard(&mut self.dctx));

        Ok(())
    }

    pub(crate) fn infer_if_expr(&mut self, ite: &HirIfExpr) -> InferResult<TyId> {
        let bool_ty = self.tctx.make_bool_ty();
        let cond_ty = self.infer_expr(&ite.cond, Some(Ty::new(bool_ty, Span::default())))?;

        self.check(
            &Ty::new(bool_ty, Span::default()),
            &Ty::new(cond_ty, ite.cond.span),
        )
        .emit(&mut self.dctx);

        let cons_ty = self.infer_block(&ite.consequent)?;
        let (alt_ty, alt_span) = ite
            .alternate
            .as_ref()
            .map(|alt| self.infer_block(&alt).map(|ty_id| (ty_id, alt.span)))
            .unwrap_or_else(|| (Ok((self.tctx.make_unit_ty(), ite.span))))?;

        self.check(
            &Ty::new(cons_ty, ite.consequent.span),
            &Ty::new(alt_ty, alt_span),
        )
        .map_err(|diag| {
            ternary!(
                ite.alternate.is_none(),
                InferDiag::error(ite.span, InferDiagErrorKind::MissingElseBranch),
                diag
            )
        })
        .emit(&mut self.dctx);

        Ok(cons_ty)
    }
}
