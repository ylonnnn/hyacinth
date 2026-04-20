use hycc_hir::path::HirPath;
use hycc_util::bug;

use crate::{ResolveResult, ty::resolver::TyResolver};

impl<'d, 'r> TyResolver<'d, 'r> {
    pub(crate) fn resolve_path(&mut self, path: &HirPath) -> ResolveResult {
        let Some(def_id) = self.resolved.get(&path.id) else {
            bug!("def id of resolved path does not exist: {:?}", path.id);
        };

        let ty_id = self.def_to_ty(*def_id)?;
        self.tctx.attach_to_hir(path.id, ty_id);

        Ok(())
    }
}
