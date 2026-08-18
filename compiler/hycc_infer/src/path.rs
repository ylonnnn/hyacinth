use hycc_hir::{def::DefSpace, path::HirPath};
use hycc_resolve::InstantiateIdent;
use hycc_ty::{
    context::TyId,
    ty::{InferKind, Ty},
};
use hycc_util::ternary;

use crate::{
    diag::{InferDiag, InferDiagErrorKind, InferResult},
    inferer::TyInferer,
};

impl<'i, 'h> TyInferer<'i, 'h> {
    pub(crate) fn infer_path(&mut self, path: &HirPath) -> InferResult<TyId> {
        let space = DefSpace::Value; // TODO

        let n = path.segments.len();
        let Some(res) = self.definitions.get_res(path.id) else {
            return Ok(self.tctx.make_inferred_ty(InferKind::Any));
        };

        let resolved_count = (n - res.unresolved);

        let mut generic_args = Vec::new();
        let mut prev_ty_id = path.segments[..resolved_count]
            .iter()
            .fold(Ok(TyId::Invalid), |_, curr| {
                self.instantiate(&mut generic_args, &curr)
            })?;

        for (i, ident) in path.segments[resolved_count..].iter().enumerate() {
            let space = ternary!(i == (n - resolved_count) - 1, space, DefSpace::Type);
            if self.definitions.get_def_id(ident.id).is_none() {
                let ty_id = prev_ty_id;
                let target = self.tctx.ext_target_kind_of(ty_id);

                let Some((_, assoc_item)) =
                    self.tctx
                        .ext_table
                        .get_assoc_item(target, space, ident.ident.ident)
                else {
                    return Err(InferDiag::error(
                        ident.span,
                        InferDiagErrorKind::UnrecognizedMember {
                            name: ident.ident.ident,
                            ty_id,
                        },
                    ));
                };

                self.definitions.define_id_hir(ident.id, assoc_item.def_id);
            };

            let definitions = &self.definitions;
            let tctx = &mut self.tctx;

            prev_ty_id = self.instantiate(&mut generic_args, &ident)?;
        }

        self.definitions.define_id_hir(
            path.id,
            self.definitions
                .expect_def_id(path.segments.last().unwrap().id),
        );

        self.tctx
            .attach_to_hir(path.id, Ty::new(prev_ty_id, path.span));

        Ok(prev_ty_id)
    }
}
