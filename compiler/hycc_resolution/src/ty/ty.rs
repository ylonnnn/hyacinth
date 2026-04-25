use hycc_hir::{
    HirMutability,
    ty::{HirTy, HirTyKind},
};
use hycc_ty::{
    context::TyId,
    ty::{RefMutability, Ty},
};
use hycc_util::ternary;

use crate::{ResolveResult, diag::{ResolverDiag, ResolverDiagErrorKind}, ty::resolver::TyResolver};

impl<'d, 'r> TyResolver<'d, 'r> {
    pub(crate) fn resolve_ty(&mut self, ty: &HirTy) -> ResolveResult<TyId> {
        let ty_id = match &ty.kind {
            HirTyKind::Path(path) => self.resolve_path(&path),
            HirTyKind::Ref(reference) => {
                let inner_ty = self.resolve_ty(&reference.ty)?;
                let mutability = ternary!(
                    reference.mutability == HirMutability::Mutable,
                    RefMutability::Mutable,
                    RefMutability::Immutable
                );

                Ok(self.tctx.make_ref_ty(inner_ty, mutability))
            }

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

        self.tctx.attach_to_hir(ty.id, Ty::new(ty_id, ty.span));
        Ok(ty_id)
    }

    pub fn resolve_as_non_inferable_ty(&mut self, ty: &HirTy) -> ResolveResult {
        let ty_id = self.resolve_ty(&ty)?;
        if !self.tctx.is_inferred(ty_id) {
            Ok(())
        } else {
            Err(Some(ResolverDiag::error(
                ty.span,
                ResolverDiagErrorKind::InvalidInference,
            )))
        }
    }
}
