use hycc_hir::path::HirPath;
use hycc_ty::context::TyId;
use hycc_util::bug;

use crate::inferer::{InferResult, TyInferer};

impl<'t, 'd, 'c, 'h> TyInferer<'t, 'd, 'c, 'h> {
    pub(crate) fn infer_path(&mut self, path: &HirPath) -> InferResult<TyId> {
        let Some(def_id) = self.definitions.get_def_id(path.id) else {
            bug!("def id of resolved path does not exist: {:?}", path.id);
        };

        let def = self.definitions.get(def_id);
        let Some(mut ty) = self.tctx.get_hir_ty(def.hir_id).cloned() else {
            bug!("hir id {:?} of def does not have a ty attached", def.hir_id);
        };

        let ty_id = (ty.id = self.tctx.resolve_ty(ty.id), ty.id).1;
        self.tctx.attach_to_hir(def.hir_id, ty);

        Ok(ty_id)
    }
}
