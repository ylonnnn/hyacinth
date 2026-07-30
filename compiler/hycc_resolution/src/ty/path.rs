use hycc_hir::path::HirPath;
use hycc_ty::{context::TyId, ty::Ty};
use hycc_util::bug;

use crate::{ResolveResult, ty::resolver::TyResolver};

impl<'t, 'd, 's> TyResolver<'t, 'd, 's> {
    pub(crate) fn resolve_path(&mut self, path: &HirPath) -> ResolveResult<TyId> {
        let Some(def_id) = self.definitions.get_def_id(path.id) else {
            bug!("def id of resolved path does not exist: {:?}", path.id);
        };

        let ty_id = self.def_to_ty(def_id, path.span)?;
        self.tctx.attach_to_hir(path.id, Ty::new(ty_id, path.span));

        Ok(ty_id)
    }
}
