use hycc_hir::ty::{HirTy, HirTyKind};
use hycc_ty::context::TyId;

use crate::{ResolveResult, ty::resolver::TyResolver};

impl<'d, 'r> TyResolver<'d, 'r> {
    pub(crate) fn resolve_ty(&mut self, ty: &HirTy) -> ResolveResult<TyId> {
        let ty_id = match &ty.kind {
            HirTyKind::Path(path) => self.resolve_path(&path),
            HirTyKind::Array(array) => {
                // TODO: construct the correct array ty
                self.resolve_ty(&array.ty)
            }

            HirTyKind::Unit(_) => Ok(self.tctx.make_unit_ty()),
        }?;

        self.tctx.attach_to_hir(ty.id, ty_id);
        Ok(ty_id)
    }
}
