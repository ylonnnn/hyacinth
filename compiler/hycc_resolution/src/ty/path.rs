use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    def::{DefKind, DefSpace},
    generic::HirGenericParamKind,
    path::{HirIdent, HirIdentArgument, HirPath},
};
use hycc_span::Span;
use hycc_ty::{
    context::TyId,
    extension::ExtTargetKind,
    ty::{GenericArg, InferKind, Ty},
};
use hycc_util::{bug, ternary};

use crate::{
    ResolveResult,
    diag::{ResolverDiag, ResolverDiagErrorKind},
    resolver_traits::{InstantiateIdent, ResolveIdentArgs},
    ty::resolver::TyResolver,
};

impl<'t, 'd, 's, 'h> InstantiateIdent<(), Option<ResolverDiag>> for TyResolver<'t, 'd, 's, 'h> {
    fn definitions(&self) -> &hycc_hir::def::DefinitionTable {
        &self.definitions
    }

    fn tctx(&mut self) -> &mut hycc_ty::context::TyCtx {
        &mut self.tctx
    }

    fn def_ty(
        &mut self,
        def_id: hycc_hir::def::DefId,
        span: Span,
    ) -> Result<TyId, Option<ResolverDiag>> {
        self.def_to_ty(def_id, span)
    }

    fn generic_arg_arity_mismatch_error(
        &self,
        span: Span,
        expected: u8,
        received: u8,
    ) -> Option<ResolverDiag> {
        Some(ResolverDiag::error(
            span,
            ResolverDiagErrorKind::GenericArgumentArityMismatch(
                ((expected as u16) << u8::BITS) | received as u16,
            ),
        ))
    }
}

impl<'t, 'd, 's, 'h> TyResolver<'t, 'd, 's, 'h> {
    pub(crate) fn resolve_path(&mut self, path: &HirPath) -> ResolveResult<TyId> {
        let n = path.segments.len();
        let res = self.definitions.expect_res(path.id);

        let mut prev_ty_id = None;
        let mut generic_args = Vec::new();

        let resolved_count = (n - res.unresolved);
        for segment in &path.segments[..resolved_count] {
            let definitions = &self.definitions;
            let tctx = &mut self.tctx;

            prev_ty_id = Some(self.instantiate(&mut generic_args, &segment)?);
        }

        for (i, ident) in path.segments[resolved_count..].iter().enumerate() {
            let space = ternary!(
                i == (n - resolved_count) - 1,
                /* TODO */ DefSpace::Value,
                DefSpace::Type
            );
            if self.definitions.get_def_id(ident.id).is_none() {
                let ty_id = prev_ty_id.unwrap();
                let target = self.tctx.ext_target_kind_of(ty_id);

                let Some((_, assoc_item)) =
                    self.tctx
                        .ext_table
                        .get_assoc_item(target, space, ident.ident.ident)
                else {
                    return Err(Some(ResolverDiag::error(
                        ident.span,
                        ResolverDiagErrorKind::UnrecognizedMember {
                            name: ident.ident.ident,
                            ty_id,
                        },
                    )));
                };

                self.definitions.define_id_hir(ident.id, assoc_item.def_id);
            };

            let definitions = &self.definitions;
            let tctx = &mut self.tctx;

            prev_ty_id = Some(self.instantiate(&mut generic_args, &ident)?);
        }

        self.definitions.define_id_hir(
            path.id,
            self.definitions
                .expect_def_id(path.segments.last().unwrap().id),
        );

        let final_ty_id = prev_ty_id.unwrap();
        self.tctx
            .attach_to_hir(path.id, Ty::new(final_ty_id, path.span));

        Ok(final_ty_id)
    }
}
