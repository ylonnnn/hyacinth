use std::sync::Arc;

use hycc_diagnostic::diagnostic::{DiagCtx, Diagnostics, FromResultEmitter};
use hycc_hir::{
    HirMutability, HirNode, HirTable,
    block::HirBlock,
    def::{
        Binding, BuiltinKind, BuiltinTyKind, DefAccessibility, DefKind, DefResolution, DefSpace,
        DefinitionTable,
    },
    expr::{
        HirArrayExpr, HirExpr, HirExprKind, HirFnCall, HirIfExpr, HirMethodCall, HirStructExpr,
        HirTupleExpr,
    },
    item::{
        HirExtend, HirFn, HirItem, HirItemKind, HirPetal, HirReferTarget, HirStruct, HirVarDecl,
    },
    path::{HirIdent, HirIdentArgument, HirPath},
    petal::PetalCtx,
    stmt::{HirStmt, HirStmtKind},
    ty::{HirTy, HirTyKind},
};
use hycc_span::Span;
use hycc_ty::{
    ctx::{TyCtx, TyId, TyResState},
    extension::ExtensionId,
    ty::{GenericArg, InferKind, RefMutability, Ty},
};
use hycc_util::ternary;

use crate::{
    InstantiateIdent, ResolveExpr, ResolveIdentArgs, ResolvePath, ResolveTy,
    diag::{ResolveResult, ResolverDiag, ResolverDiagCtx, ResolverDiagErrorKind},
};

#[derive(Debug)]
pub struct TyResolver<'t, 'h> {
    pub dctx: ResolverDiagCtx<'t>,
    pub tctx: &'t mut TyCtx,
    pub definitions: &'t mut DefinitionTable,
    pub hir_table: &'t HirTable<'h>,
    pub petal_ctx: &'t mut PetalCtx,

    expected_space: Option<DefSpace>,
}

impl<'t, 'h> TyResolver<'t, 'h> {
    pub fn new(
        dctx: &'t mut DiagCtx,
        tctx: &'t mut TyCtx,
        definitions: &'t mut DefinitionTable,
        hir_table: &'t HirTable<'h>,
        petal_ctx: &'t mut PetalCtx,
    ) -> Self {
        Self {
            dctx: ResolverDiagCtx::new(dctx),
            tctx,
            definitions,
            hir_table,
            petal_ctx,

            expected_space: None,
        }
    }

    fn expect_space<U>(&mut self, space: DefSpace, mut f: impl FnMut(&mut Self) -> U) -> U {
        let prev_space = self.expected_space;
        self.expected_space.replace(space);

        let data = f(self);
        self.expected_space = prev_space;

        data
    }

    pub fn resolve(&mut self, tree: &HirItem) {
        self.resolve_item(&tree);
    }

    fn resolve_item(&mut self, item: &HirItem) -> ResolveResult {
        match &item.kind {
            HirItemKind::Refer(_) => Ok(()),
            HirItemKind::Petal(petal) => self.resolve_petal(&petal),
            HirItemKind::Proto(_) => todo!("resolve proto"),
            HirItemKind::Extend(_) => Ok(()),
            HirItemKind::Struct(strct) => self.resolve_struct(&strct),
            HirItemKind::Fn(_) => self.resolve_fn(&item),
            HirItemKind::VarDecl(_) => self.resolve_var_decl(&item),
        }
    }

    fn resolve_petal(&mut self, petal: &HirPetal) -> ResolveResult {
        let petals = petal
            .path()
            .map(|path| {
                path.segments
                    .iter()
                    .map(|segment| {
                        self.petal_ctx
                            .expect_def_petal_id(self.definitions.expect_def_id(segment.id))
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec![self.petal_ctx.root_petal_id()]);

        petals
            .iter()
            .for_each(|petal_id| self.petal_ctx.push(*petal_id));

        petal
            .items
            .iter()
            .for_each(|item| self.resolve_item(&item).emit_discard(&mut self.dctx));

        (0..petals.len()).for_each(|_| self.petal_ctx.pop());

        Ok(())
    }

    fn resolve_native_exts(&mut self) {
        for hir_id in self.tctx.ext_table.hir_ids() {
            let HirNode::Item(item) = self.hir_table.get(hir_id) else {
                unreachable!()
            };

            self.resolve_extend(&item).emit(&mut self.dctx);
        }

        let exts = self.tctx.ext_table.ext_ids();
        exts.iter().enumerate().for_each(|(i, base)| {
            &exts[(i + 1)..].iter().for_each(|target| {
                let (base_ext, target_ext) = (
                    self.tctx.ext_table.get(*base),
                    self.tctx.ext_table.get(*target),
                );
                let (base_raw_ty_id, target_raw_ty_id) =
                    (base_ext.expect_target(), target_ext.expect_target());
                let (b_gp_count, t_gp_count) =
                    (base_ext.generic_param_count, target_ext.generic_param_count);

                let (base_generic_args, target_generic_args) = (
                    (0..b_gp_count)
                        .map(|_| {
                            GenericArg::Ty(
                                self.tctx.make_inferred_ty(Span::default(), InferKind::Any),
                            )
                        })
                        .collect::<Arc<_>>(),
                    (0..t_gp_count)
                        .map(|_| {
                            GenericArg::Ty(
                                self.tctx.make_inferred_ty(Span::default(), InferKind::Any),
                            )
                        })
                        .collect::<Arc<_>>(),
                );
                let (base_ty_id, target_ty_id) = (
                    self.tctx.instantiate(base_raw_ty_id, &[&base_generic_args]),
                    self.tctx
                        .instantiate(target_raw_ty_id, &[&target_generic_args]),
                );

                if !self.tctx.unify_ty(base_ty_id, target_ty_id) {
                    return;
                }

                self.tctx
                    .ext_table
                    .get(*base)
                    .collisions(self.tctx.ext_table.get(*target))
                    .iter()
                    .for_each(|((_, name), prev_def, redef)| {
                        let def = self.definitions.get(redef.def_id);
                        self.dctx.error(
                            def.span,
                            ResolverDiagErrorKind::Duplication {
                                ident: *name,
                                earlier_def: prev_def.def_id,
                            },
                        )
                    });
            });
        });
    }

    fn resolve_extend(&mut self, item: &HirItem) -> ResolveResult {
        let extend = item.expect_extend();

        let target_ty_id = self.resolve_ty(&extend.target)?;
        let ext_id = self.tctx.ext_table.expect_hir_ext_id(item.id);
        self.tctx
            .ext_table
            .get_mut(ext_id)
            .attach_target(target_ty_id);

        if !self.tctx.is_error_ty(target_ty_id) {
            let target_petal = self
                .tctx
                .get_ty_def_id(target_ty_id)
                .and_then(|def_id| self.definitions.get(def_id).petal);

            let target_kind = self.tctx.ext_target_kind_of(target_ty_id);
            self.tctx.ext_table.attach_id(target_kind, ext_id);

            self.tctx
                .ext_table
                .get_mut(ext_id)
                .items
                .iter()
                .for_each(|(_, binding)| {
                    let def = self.definitions.get_mut(binding.def_id);
                    def.petal = target_petal;
                });
        }

        extend
            .items
            .iter()
            .for_each(|item| self.resolve_item(&item).emit_discard(&mut self.dctx));

        Ok(())
    }

    fn resolve_struct(&mut self, strct: &HirStruct) -> ResolveResult {
        strct
            .fields
            .list
            .iter()
            .for_each(|field| self.resolve_ty(&field.ty).emit_discard(&mut self.dctx));

        Ok(())
    }

    fn resolve_fn(&mut self, item: &HirItem) -> ResolveResult {
        let func = item.expect_fn();
        // let def_id = self.definitions.expect_def_id(item.id);

        let generic_args = func.sig.generic_params.as_ref().map_or_else(
            || Vec::new(),
            |generic_params| {
                generic_params
                    .list
                    .iter()
                    .map(|generic_param| {
                        GenericArg::Ty(self.tctx.expect_hir_ty_id(generic_param.id))
                    })
                    .collect::<Vec<_>>()
            },
        );

        let params = func
            .sig
            .params
            .list
            .iter()
            .filter_map(|param| {
                let ty_id = self.resolve_ty(&param.ty).emit(&mut self.dctx)?;
                self.tctx
                    .attach_to_hir(param.id, Ty::new(ty_id, param.span));
                Some(ty_id)
            })
            .collect::<Arc<_>>();

        let unit_ty = self.tctx.make_unit_ty();
        let ret_ty = func.sig.ret_ty.as_ref().map_or(unit_ty, |ret_ty| {
            let ty_id = self.resolve_ty(&ret_ty).emit(&mut self.dctx).unwrap();

            self.tctx
                .attach_to_hir(ret_ty.id, Ty::new(ty_id, ret_ty.span));
            ty_id
        });

        let n_fn_ty_id = self.tctx.make_fn_ty(
            generic_args.into(),
            self.definitions.get_def_id(item.id),
            params.into(),
            ret_ty,
        );

        let fn_ty_id = self.tctx.expect_hir_ty_id(item.id);
        self.tctx.unify_ty(fn_ty_id, n_fn_ty_id);

        self.tctx
            .attach_to_hir(item.id, Ty::new(n_fn_ty_id, item.span));
        // self.tctx.attach_to_def(def_id, fn_ty);

        if self.tctx.is_inferred(n_fn_ty_id) {
            self.tctx
                .update_hir_res_state(item.id, TyResState::Unresolved);
        }

        self.resolve_block(&func.body).emit(&mut self.dctx);

        Ok(())
    }

    fn resolve_var_decl(&mut self, item: &HirItem) -> ResolveResult {
        let decl = item.expect_var();

        let var_ty_id = self.tctx.expect_hir_ty_id(item.id);
        let (ty_id, span) = decl.ty.as_ref().map_or_else(
            || Ok((var_ty_id.clone(), item.span)),
            |ty| self.resolve_ty(ty).map(|ty_id| (ty_id, ty.span)),
        )?;

        if self.definitions.get_def_id(item.id).is_some() {
            self.tctx.attach_to_hir(item.id, Ty::new(ty_id, span));

            if item.is_top_level() && self.tctx.is_inferred(ty_id) {
                self.tctx
                    .update_hir_res_state(item.id, TyResState::Unresolved);
            }
        }

        decl.val
            .as_ref()
            .map(|expr| self.resolve_expr(&expr).emit(&mut self.dctx));

        Ok(())
    }

    fn resolve_block(&mut self, block: &HirBlock) -> ResolveResult {
        block
            .stmts
            .iter()
            .for_each(|stmt| self.resolve_stmt(&stmt).emit_discard(&mut self.dctx));

        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &HirStmt) -> ResolveResult {
        match &stmt.kind {
            HirStmtKind::Ret(ret) => ret
                .value
                .map_or_else(|| Ok(()), |val| self.resolve_expr(&val)),
            HirStmtKind::Pass(pass) => pass
                .value
                .map_or_else(|| Ok(()), |val| self.resolve_expr(&val)),

            HirStmtKind::Item(item) => self.resolve_item(&item),
            HirStmtKind::Expr(expr) => self.resolve_expr(&expr),
        }
    }

    fn resolve_binary_expr(&mut self, left: &HirExpr, right: &HirExpr) -> ResolveResult {
        self.resolve_expr(&left).emit(&mut self.dctx);
        self.resolve_expr(&right)
    }

    fn resolve_assign_expr(&mut self, assignee: &HirExpr, expr: &HirExpr) -> ResolveResult {
        self.resolve_expr(&assignee).emit(&mut self.dctx);
        self.resolve_expr(&expr)
    }

    fn resolve_array_expr(&mut self, array: &HirArrayExpr) -> ResolveResult {
        array
            .elements
            .iter()
            .for_each(|el| self.resolve_expr(&el).emit_discard(&mut self.dctx));

        Ok(())
    }

    fn resolve_tuple_expr(&mut self, tup: &HirTupleExpr) -> ResolveResult {
        tup.elements
            .iter()
            .for_each(|el| self.resolve_expr(&el).emit_discard(&mut self.dctx));

        Ok(())
    }

    fn resolve_struct_expr(&mut self, strct: &HirStructExpr) -> ResolveResult {
        self.expect_space(DefSpace::Type, |s| {
            s.resolve_path(&strct.path).emit(&mut s.dctx)
        });

        strct
            .fields
            .iter()
            .for_each(|field| self.resolve_expr(&field.val).emit_discard(&mut self.dctx));

        Ok(())
    }

    fn resolve_anon_fn_expr(&mut self, anfn_expr: &HirExpr) -> ResolveResult {
        let HirExprKind::AnonFn(anfn) = &anfn_expr.kind else {
            unreachable!()
        };

        let params = anfn
            .params
            .list
            .iter()
            .map(|param| {
                let p_ty_id = param
                    .ty
                    .as_ref()
                    .and_then(|ty| self.resolve_ty(&ty).emit(&mut self.dctx))
                    .unwrap_or_else(|| self.tctx.make_inferred_ty(param.span, InferKind::Any));

                self.tctx.attach_to_hir(
                    param.id,
                    Ty::new(p_ty_id, param.ty.map(|ty| ty.span).unwrap_or(param.span)),
                );

                p_ty_id
            })
            .collect::<Arc<_>>();

        let ret_ty = anfn
            .ret_ty
            .as_ref()
            .and_then(|ty| self.resolve_ty(&ty).emit(&mut self.dctx))
            .unwrap_or_else(|| self.tctx.make_inferred_ty(anfn.body.span, InferKind::Any));

        let fn_ty = self
            .tctx
            .make_fn_ty(Arc::new([]), None, params.into(), ret_ty);
        self.tctx
            .attach_to_hir(anfn_expr.id, Ty::new(fn_ty, anfn_expr.span));

        self.resolve_block(&anfn.body)
    }

    fn resolve_fn_call_expr(&mut self, call: &HirFnCall) -> ResolveResult {
        self.resolve_expr(&call.callee).emit(&mut self.dctx);

        call.arguments
            .data
            .iter()
            .for_each(|argument| self.resolve_expr(&argument).emit_discard(&mut self.dctx));

        Ok(())
    }

    fn resolve_method_call_expr(&mut self, call: &HirMethodCall) -> ResolveResult {
        self.resolve_expr(&call.receiver).emit(&mut self.dctx);

        call.callee.arguments.as_ref().map(|arguments| {
            arguments.data.iter().for_each(|argument| match &argument {
                HirIdentArgument::Ty(ty) => self.resolve_ty(&ty).emit_discard(&mut self.dctx),
                HirIdentArgument::Expr(expr) => todo!("const generic args"),
            });
        });

        call.arguments
            .data
            .iter()
            .for_each(|argument| self.resolve_expr(&argument).emit_discard(&mut self.dctx));

        Ok(())
    }

    fn resolve_if_expr(&mut self, ite: &HirIfExpr) -> ResolveResult {
        self.resolve_expr(&ite.cond).emit(&mut self.dctx);
        self.resolve_block(&ite.consequent).emit(&mut self.dctx);
        ite.alternate
            .as_ref()
            .map(|alt| self.resolve_block(&alt).emit(&mut self.dctx));

        Ok(())
    }
}

impl<'t, 'h> ResolveExpr<(), ResolverDiag> for TyResolver<'t, 'h> {
    fn resolve_expr(&mut self, expr: &HirExpr) -> ResolveResult {
        self.expect_space(DefSpace::Value, |s| match &expr.kind {
            HirExprKind::Path(path) => {
                // Check the resolution state of the path if there are any
                // unresolved segments, and if so, resolve extensions
                if s.definitions.expect_res(path.id).unresolved > 0 {
                    s.preprocessor();
                }

                // Resolve the arguments of each segment only
                path.segments
                    .iter()
                    .filter_map(|segment| {
                        segment
                            .arguments
                            .as_ref()
                            .map(|arguments| s.resolve_ident_args(&arguments))
                    })
                    .collect::<Result<Vec<_>, _>>();

                Ok(())
            }

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
}

impl<'t, 'h> ResolveTy<ResolverDiag> for TyResolver<'t, 'h> {
    fn resolve_ty(&mut self, ty: &HirTy) -> ResolveResult<TyId> {
        let ty_id = self.expect_space(DefSpace::Type, |s| -> ResolveResult<TyId> {
            match &ty.kind {
                HirTyKind::Unit(_) => Ok(s.tctx.make_unit_ty()),

                HirTyKind::Path(path) => s.resolve_path(&path),
                HirTyKind::Ref(reference) => {
                    let inner_ty = s.resolve_ty(&reference.ty)?;
                    let mutability = ternary!(
                        reference.mutability == HirMutability::Mutable,
                        RefMutability::Mutable,
                        RefMutability::Immutable
                    );

                    Ok(s.tctx.make_ref_ty(inner_ty, mutability))
                }

                HirTyKind::Array(array) => {
                    // TODO: construct the correct array ty
                    let ty_id = s.resolve_ty(&array.ty)?;
                    Ok(s.tctx.make_array_ty(ty_id))
                }

                HirTyKind::Slice(slice) => {
                    let ty_id = s.resolve_ty(&slice.ty)?;
                    Ok(s.tctx.make_slice_ty(ty_id))
                }

                HirTyKind::Tuple(tup) => {
                    let tys = tup
                        .data
                        .iter()
                        .filter_map(|el| s.resolve_ty(&el).emit(&mut s.dctx))
                        .collect::<Arc<_>>();
                    Ok(s.tctx.make_tuple_ty(tys))
                }

                HirTyKind::Fn(func) => {
                    let params = func
                        .params
                        .iter()
                        .filter_map(|param| s.resolve_ty(&param).emit(&mut s.dctx))
                        .collect::<Arc<_>>();

                    let ret_ty_id = func
                        .ret_ty
                        .as_ref()
                        .and_then(|ret_ty| s.resolve_ty(&ret_ty).emit(&mut s.dctx))
                        .unwrap_or_else(|| s.tctx.make_unit_ty());

                    Ok(s.tctx
                        .make_fn_ty(Arc::new([]), None, params.into(), ret_ty_id))
                }
            }
        })?;

        self.tctx.attach_to_hir(ty.id, Ty::new(ty_id, ty.span));
        Ok(ty_id)
    }
}

impl<'t, 'h> ResolveIdentArgs<(), ResolverDiag> for TyResolver<'t, 'h> {}

impl<'t, 'h> InstantiateIdent<(), ResolverDiag> for TyResolver<'t, 'h> {
    fn definitions(&self) -> &DefinitionTable {
        &self.definitions
    }

    fn definitions_mut(&mut self) -> &mut DefinitionTable {
        &mut self.definitions
    }

    fn tctx(&mut self) -> &mut TyCtx {
        &mut self.tctx
    }

    fn def_ty(
        &mut self,
        def_id: hycc_hir::def::DefId,
        span: hycc_span::Span,
    ) -> ResolveResult<TyId> {
        let def = self.definitions.get(def_id);
        let ty_id = match &def.kind {
            DefKind::Petal => Err(ResolverDiag::error(
                span,
                ResolverDiagErrorKind::IllegalPetalTyUsage(def_id),
            ))?,

            DefKind::Builtin(BuiltinKind::Ty(kind)) => match &kind {
                BuiltinTyKind::Infer => self.tctx.make_inferred_ty(span, InferKind::Any),
                _ => self.tctx.expect_def_ty_id(def_id),
            },

            _ => self.tctx.expect_hir_ty_id(def.hir_id),
        };

        Ok(ty_id)
    }

    fn generic_arg_arity_mismatch_error(
        &self,
        span: hycc_span::Span,
        expected: u8,
        received: u8,
    ) -> ResolverDiag {
        ResolverDiag::error(
            span,
            ResolverDiagErrorKind::GenericArgumentArityMismatch(
                ((expected as u16) << u8::BITS) | received as u16,
            ),
        )
    }
}

impl<'t, 'h> ResolvePath<(), ResolverDiag> for TyResolver<'t, 'h> {
    fn expected_space(&self) -> Option<DefSpace> {
        self.expected_space
    }

    fn petal_ctx(&self) -> &hycc_hir::petal::PetalCtx {
        &self.petal_ctx
    }

    fn unrecognized_member_error(
        &self,
        span: Span,
        name: hycc_symbol::Symbol,
        ty_id: TyId,
    ) -> ResolverDiag {
        ResolverDiag::error(
            span,
            ResolverDiagErrorKind::UnrecognizedMember { name, ty_id },
        )
    }

    fn inaccessible_error(
        &self,
        span: Span,
        name: hycc_symbol::Symbol,
        kind: Option<crate::diag::SymbolKind>,
    ) -> ResolverDiag {
        ResolverDiag::error(span, ResolverDiagErrorKind::Inaccessible(name, kind))
    }

    fn multiple_assoc_item_matched_error(
        &self,
        span: Span,
        name: hycc_symbol::Symbol,
        matches: Vec<(ExtensionId, Binding)>,
    ) -> ResolverDiag {
        ResolverDiag::error(
            span,
            ResolverDiagErrorKind::MultipleAssocItemsMatched(name, matches),
        )
    }

    fn preprocessor(&mut self) {
        if !self.tctx.ext_table.native_exts_resolved() {
            self.resolve_native_exts();
        }
    }
}
