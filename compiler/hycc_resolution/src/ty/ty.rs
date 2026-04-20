use hycc_hir::ty::{HirTy, HirTyKind};

use crate::{ResolveResult, ty::resolver::TyResolver};

impl<'d, 'r> TyResolver<'d, 'r> {
    pub(crate) fn resolve_ty(&mut self, ty: &HirTy) -> ResolveResult {
        match &ty.kind {
            HirTyKind::Path(path) => self.resolve_path(&path),
            HirTyKind::Array(array) => self.resolve_ty(&array.ty),
            HirTyKind::Unit(_) => Ok(()),
        }
    }
}
