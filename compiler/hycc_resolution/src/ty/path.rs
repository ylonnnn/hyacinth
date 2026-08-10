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
use hycc_util::bug;

use crate::{
    ResolveResult,
    diag::{ResolverDiag, ResolverDiagErrorKind},
    ty::resolver::TyResolver,
};

impl<'t, 'd, 's, 'h> TyResolver<'t, 'd, 's, 'h> {
    pub(crate) fn instantiate_segment(
        &mut self,
        segment: &HirIdent,
        generic_args: &mut Vec<Vec<GenericArg>>,
    ) -> ResolveResult<TyId> {
        let def_id = self.definitions.expect_def_id(segment.id);

        let mut g_args = Vec::new();
        if let Some(arguments) = &segment.arguments {
            for argument in &arguments.data {
                match argument {
                    HirIdentArgument::Ty(ty) => {
                        g_args.push(GenericArg::Ty(self.resolve_ty(ty)?));
                    }
                    HirIdentArgument::Expr(_expr) => {
                        // todo: const generics -> GenericArg::Const
                        todo!("const generic args")
                    }
                }
            }
        }

        let generic_params = self.definitions.get(def_id).generic_params().unwrap_or(&[]);
        let generic_param_count = generic_params.len();

        let n = g_args.len();
        if n > generic_param_count {
            todo!(
                "throw error: generic argument arity mismatch. expected: <={:?}, received: {:?}",
                generic_param_count,
                n
            )
        }

        for i in n..generic_param_count {
            let gp_def_id = generic_params[i];
            let gp_def = self.definitions.get(gp_def_id).kind.expect_generic_param();

            g_args.push(match &gp_def.kind {
                HirGenericParamKind::Ty => {
                    GenericArg::Ty(self.tctx.make_inferred_ty(InferKind::Any))
                }

                HirGenericParamKind::Const => todo!("const generic arg"),
            });
        }

        if !g_args.is_empty() {
            generic_args.push(g_args);
        }

        let raw_ty_id = self.def_to_ty(def_id, segment.span)?;
        let ty_id = self.tctx.instantiate(
            raw_ty_id,
            &generic_args
                .iter()
                .map(|args| args.iter().as_slice())
                .collect::<Vec<_>>(),
        );

        self.tctx
            .attach_to_hir(segment.id, Ty::new(ty_id, segment.span));
        Ok(ty_id)
    }

    pub(crate) fn resolve_path(&mut self, path: &HirPath) -> ResolveResult<TyId> {
        let n = path.segments.len();
        let res = self.definitions.expect_res(path.id);

        let mut prev_ty_id = None;
        let mut generic_args = Vec::new();

        let resolved_count = (n - res.unresolved);
        for segment in &path.segments[..resolved_count] {
            prev_ty_id = Some(self.instantiate_segment(&segment, &mut generic_args)?);
        }

        for ident in &path.segments[resolved_count..] {
            if self.definitions.get_def_id(ident.id).is_none() {
                let ty_id = prev_ty_id.unwrap();
                let target = self.tctx.ext_target_kind_of(ty_id);

                let Some((_, assoc_item)) = self.tctx.ext_table.get_assoc_item(
                    target,
                    DefSpace::Value, /* TODO */
                    ident.ident.ident,
                ) else {
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

            prev_ty_id = Some(self.instantiate_segment(&ident, &mut generic_args)?);
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
