use hycc_hir::path::HirPath;
use hycc_ty::context::TyId;
use hycc_util::bug;

use crate::inferer::{InferResult, TyInferer};

impl<'t, 'd, 'r> TyInferer<'t, 'd, 'r> {
    pub(crate) fn infer_path(&mut self, path: &HirPath) -> InferResult<TyId> {
        let Some(def_id) = self.resolved.get(&path.id) else {
            bug!("def id of resolved path does not exist: {:?}", path.id);
        };

        let def = self.definitions.get(*def_id);
        let Some(ty_id) = self.tctx.get_ty_of_hir(def.hir_id) else {
            bug!("hir id {:?} of def does not have a ty_id attached", def.hir_id);
        };

        Ok(ty_id)
    }
}
