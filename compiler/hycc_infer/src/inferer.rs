use std::collections::HashSet;

use hycc_const::table::ConstTable;
use hycc_diagnostic::diagnostic::{DiagCtx, Diagnostics};
use hycc_hir::{
    HirNode, HirTable,
    def::DefinitionTable,
    item::{HirItem, HirItemKind},
    petal::PetalCtx,
};
use hycc_resolve::{InstantiateIdent, ResolveExpr, ResolveIdentArgs, ResolveTy};
use hycc_span::Span;
use hycc_ty::{
    ctx::{TyCtx, TyId, TyVarId},
    ty::{InferKind, IntTy, Ty, TyKind},
};
use hycc_util::{bug, ternary};

use crate::{
    diag::{InferDiag, InferDiagCtx, InferDiagErrorKind},
    fn_ctx::FnCtx,
};

#[derive(Debug)]
pub struct TyInferer<'i, 'h> {
    pub dctx: InferDiagCtx<'i>,
    pub(crate) fn_ctx: Option<FnCtx>,

    pub tctx: &'i mut TyCtx,
    pub definitions: &'i mut DefinitionTable,
    pub(crate) const_table: &'i ConstTable,
    pub(crate) hir_table: &'i HirTable<'h>,
    pub(crate) petal_ctx: &'i PetalCtx,
}

impl<'i, 'h> TyInferer<'i, 'h> {
    pub fn new(
        dctx: &'i mut DiagCtx,
        tctx: &'i mut TyCtx,
        definitions: &'i mut DefinitionTable,
        const_table: &'i ConstTable,
        hir_table: &'i HirTable<'h>,
        petal_ctx: &'i PetalCtx,
    ) -> Self {
        Self {
            dctx: InferDiagCtx::new(dctx),
            fn_ctx: None,

            tctx,
            definitions,
            const_table,
            hir_table,
            petal_ctx,
        }
    }

    pub fn compatible(&mut self, expected: TyId, received: TyId) -> bool {
        self.tctx.unify_ty(expected, received)
    }

    pub fn check(&mut self, expected: &Ty, received: &Ty) -> Option<InferDiag> {
        if self.compatible(expected.id, received.id) {
            None
        } else {
            Some(InferDiag::error(
                received.span,
                InferDiagErrorKind::TypeMismatch {
                    expectation_span: expected.span,
                    expected: self.tctx.resolve_ty(expected.id),
                    received: self.tctx.resolve_ty(received.id),
                },
            ))
        }
    }

    pub fn use_fn_ctx<F, U>(&mut self, ctx: FnCtx, mut handler: F) -> U
    where
        F: FnMut(&mut Self) -> U,
    {
        let prev_ctx = self.fn_ctx.take();
        self.fn_ctx.replace(ctx);

        let data = handler(self);
        self.fn_ctx = prev_ctx;

        data
    }

    pub(crate) fn analyze_unresolved(&mut self) {
        let emit_err = !*self.dctx.error_flag();

        let Some(fn_ctx) = &self.fn_ctx else {
            bug!("unresolved type analysis can only be performed within a function body")
        };

        let HirNode::Block(block) = self.hir_table.get(fn_ctx.fn_body) else {
            unreachable!()
        };

        let default_int_ty_id = self.tctx.make_int_ty(IntTy::Fixed(32, true));
        let default_float_ty_id = self.tctx.make_float_ty(32);

        let mut seen = HashSet::new();

        for stmt in &block.stmts {
            let Some(ty_id) = self.tctx.get_hir_ty_id(stmt.id) else {
                continue;
            };

            let mut unresolved_tys = Vec::new();
            self.tctx.unresolved_infer(ty_id, &mut unresolved_tys);

            for unresolved_ty_id in unresolved_tys {
                let TyKind::Infer(var_id, kind) = self.tctx.get(unresolved_ty_id) else {
                    continue;
                };

                match &kind {
                    InferKind::Int => self.tctx.bind_var(*var_id, default_int_ty_id),
                    InferKind::Float => self.tctx.bind_var(*var_id, default_float_ty_id),
                    _ => {
                        if seen.insert(*var_id) && emit_err {
                            let span = self.tctx.get_var(*var_id).span;
                            self.dctx.error(
                                span,
                                InferDiagErrorKind::UnresolvedTy(Ty::new(
                                    self.tctx.intern(TyKind::Infer(*var_id, *kind)),
                                    span,
                                )),
                            );
                        }

                        continue;
                    }
                }
            }
        }

        // let mut tys = self.tctx.unresolved_tys();
        // tys.sort_by(|(a_var_id, _), (b_var_id, _)| {
        //     let (a_var, b_var) = (self.tctx.get_var(*a_var_id), self.tctx.get_var(*b_var_id));
        //     a_var.span.offset.cmp(&b_var.span.offset)
        // });

        // for (var_id, kind) in &tys {
        //     match &kind {
        //         InferKind::Int => self.tctx.bind_var(*var_id, default_int_ty_id),
        //         InferKind::Float => self.tctx.bind_var(*var_id, default_float_ty_id),
        //         _ => {
        //             if emit_err {
        //                 let span = self.tctx.get_var(*var_id).span;
        //                 self.dctx.error(
        //                     span,
        //                     InferDiagErrorKind::UnresolvedTy(Ty::new(
        //                         self.tctx.intern(TyKind::Infer(*var_id, *kind)),
        //                         span,
        //                     )),
        //                 );
        //             }

        //             continue;
        //         }
        //     };
        // }
    }

    pub fn infer(&mut self, tree: &HirItem) {
        let petal = tree.expect_petal();
        self.infer_petal(&petal);

        // let err = *self.dctx.error_flag();
        // self.analyze_unresolved(!err);
    }
}

impl<'i, 'h> ResolveTy<InferDiag> for TyInferer<'i, 'h> {
    fn resolve_ty(&mut self, ty: &hycc_hir::ty::HirTy) -> Result<TyId, InferDiag> {
        Ok(self.tctx.expect_hir_ty_id(ty.id))
    }
}

impl<'i, 'h> ResolveIdentArgs<TyId, InferDiag> for TyInferer<'i, 'h> {}

impl<'i, 'h> InstantiateIdent<TyId, InferDiag> for TyInferer<'i, 'h> {
    fn definitions(&self) -> &DefinitionTable {
        &self.definitions
    }

    fn tctx(&mut self) -> &mut TyCtx {
        &mut self.tctx
    }

    fn def_ty(
        &mut self,
        def_id: hycc_hir::def::DefId,
        _span: hycc_span::Span,
    ) -> Result<TyId, InferDiag> {
        let def = self.definitions.get(def_id);
        Ok(self.tctx.expect_hir_ty_id(def.hir_id))
    }

    fn generic_arg_arity_mismatch_error(
        &self,
        span: Span,
        expected: u8,
        received: u8,
    ) -> InferDiag {
        InferDiag::error(
            span,
            InferDiagErrorKind::GenericArgumentArityMismatch(
                ((expected as u16) << u8::BITS) | received as u16,
            ),
        )
    }
}
