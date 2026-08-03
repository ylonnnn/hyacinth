use hycc_hir::{
    def::DefKind,
    generic::HirGenericParamKind,
    path::{HirIdentArgument, HirPath},
};
use hycc_ty::{
    context::TyId,
    ty::{GenericArg, InferKind, Ty},
};
use hycc_util::bug;

use crate::{
    ResolveResult,
    diag::{ResolverDiag, ResolverDiagErrorKind},
    ty::resolver::TyResolver,
};

impl<'t, 'd, 's> TyResolver<'t, 'd, 's> {
    pub(crate) fn resolve_path(&mut self, path: &HirPath) -> ResolveResult<TyId> {
        let mut prev_ty_id = None;

        for ident in &path.segments {
            let Some(def_id) = self.definitions.get_def_id(ident.id) else {
                bug!("def id of resolved segment does not exist: {:?}", ident.id);
            };

            let mut generic_args = Vec::new();
            if let Some(arguments) = &ident.arguments {
                for argument in &arguments.data {
                    match argument {
                        HirIdentArgument::Ty(ty) => {
                            let arg_ty_id = self.resolve_ty(ty)?;
                            generic_args.push(GenericArg::Ty(arg_ty_id));
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

            let n = generic_args.len();
            if n > generic_param_count {
                todo!("throw error: generic argument arity mismatch")
            }

            for i in n..generic_param_count {
                let gp_def_id = generic_params[i];
                let def = self.definitions.get(gp_def_id);

                let DefKind::GenericParam(gp_def) = &def.kind else {
                    unreachable!()
                };

                generic_args.push(match &gp_def.kind {
                    HirGenericParamKind::Ty => {
                        GenericArg::Ty(self.tctx.make_inferred_ty(InferKind::Any))
                    }

                    HirGenericParamKind::Const => todo!("const generic arg"),
                });
            }

            let raw_ty_id = self.def_to_ty(def_id, ident.span)?;
            let ty_id = self.tctx.instantiate(raw_ty_id, generic_args.into());

            prev_ty_id = Some(ty_id);
            self.tctx
                .attach_to_hir(ident.id, Ty::new(ty_id, ident.span));
        }

        let final_ty_id = prev_ty_id.unwrap();
        self.tctx
            .attach_to_hir(path.id, Ty::new(final_ty_id, path.span));

        Ok(final_ty_id)
    }
}
