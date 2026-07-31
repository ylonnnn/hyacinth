use hycc_hir::path::HirPath;
use hycc_ty::context::TyId;
use hycc_util::bug;

use crate::inferer::{InferResult, TyInferer};

impl<'t, 'd, 'c, 'h, 'p> TyInferer<'t, 'd, 'c, 'h, 'p> {
    pub(crate) fn infer_path(&mut self, path: &HirPath) -> InferResult<TyId> {
        let def_id = self.definitions.expect_def_id(path.id);
        let def = self.definitions.get(def_id);

        let Some(mut ty) = self
            .tctx
            .get_hir_ty(def.hir_id)
            .or_else(|| self.tctx.get_ty_of_def(def_id))
            .cloned()
        else {
            bug!(
                "no type attached to definition {def_id:?} nor its hir {:?}",
                def.hir_id
            )
        };

        let ty_id = (ty.id = self.tctx.resolve_ty(ty.id), ty.id).1;
        // self.tctx.attach_to_hir(def.hir_id, ty);

        Ok(ty_id)
    }
}
