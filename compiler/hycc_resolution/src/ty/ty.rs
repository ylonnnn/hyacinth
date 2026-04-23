use hycc_hir::ty::{HirTy, HirTyKind};
use hycc_ty::context::TyId;

use crate::{ResolveResult, ty::resolver::TyResolver};

impl<'d, 'r> TyResolver<'d, 'r> {
    pub(crate) fn resolve_ty(&mut self, ty: &HirTy) -> ResolveResult<TyId> {
        let ty_id = match &ty.kind {
            HirTyKind::Path(path) => self.resolve_path(&path),

            HirTyKind::Array(array) => {
                // TODO: construct the correct array ty
                let ty_id = self.resolve_ty(&array.ty)?;
                Ok(self.tctx.make_array_ty(ty_id))
            }

            HirTyKind::Slice(slice) => {
                // TODO: construct the correct slice ty
                let ty_id = self.resolve_ty(&slice.ty)?;
                Ok(self.tctx.make_slice_ty(ty_id))
            }

            HirTyKind::Unit(_) => Ok(self.tctx.make_unit_ty()),
        }?;

        self.tctx.attach_to_hir(ty.id, ty_id);
        Ok(ty_id)
    }
}
