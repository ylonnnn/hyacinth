use std::sync::Arc;

use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    HirMutability,
    scope::Scope,
    ty::{HirTy, HirTyKind},
};
use hycc_ty::{
    context::TyId,
    ty::{RefMutability, Ty},
};
use hycc_util::ternary;

use crate::{
    ResolveResult,
    diag::{ResolverDiag, ResolverDiagErrorKind},
    resolver_traits::{ResolveIdentArgs, ResolveTy},
    ty::resolver::TyResolver,
};

impl<'t, 'd, 's, 'h> ResolveTy<Option<ResolverDiag>> for TyResolver<'t, 'd, 's, 'h> {
    fn resolve_ty(
        &mut self,
        ty: &hycc_hir::ty::HirTy,
    ) -> Result<hycc_ty::context::TyId, Option<ResolverDiag>> {
        let ty_id = match &ty.kind {
            HirTyKind::Unit(_) => Ok(self.tctx.make_unit_ty()),

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

            HirTyKind::Tuple(tup) => {
                let mut tys = Vec::new();
                for el in &tup.data {
                    match self.resolve_ty(&el) {
                        Ok(ty_id) => tys.push(ty_id),
                        Err(diag) => {
                            if let Some(diag) = diag {
                                self.dctx.add(diag);
                            }

                            continue;
                        }
                    }
                }

                Ok(self.tctx.make_tuple_ty(tys.into()))
            }

            HirTyKind::Fn(func) => {
                let mut params = Vec::new();
                for param in &func.params {
                    match self.resolve_ty(&param) {
                        Ok(ty_id) => params.push(ty_id),
                        Err(diag) => {
                            if let Some(diag) = diag {
                                self.dctx.add(diag);
                            }

                            continue;
                        }
                    }
                }

                let mut ret_ty: TyId = self.tctx.make_unit_ty();
                if let Some(r_ty) = func.ret_ty {
                    match self.resolve_ty(&r_ty) {
                        Ok(ty_id) => ret_ty = ty_id,
                        Err(diag) => {
                            if let Some(diag) = diag {
                                self.dctx.add(diag);
                            }
                        }
                    }
                }

                Ok(self
                    .tctx
                    .make_fn_ty(Arc::new([]), None, params.into(), ret_ty))
            }
        }?;

        self.tctx.attach_to_hir(ty.id, Ty::new(ty_id, ty.span));
        Ok(ty_id)
    }
}

impl<'t, 'd, 's, 'h> TyResolver<'t, 'd, 's, 'h> {
    // pub(crate) fn resolve_ty(&mut self, ty: &HirTy) -> ResolveResult<TyId> {
    //     let ty_id = match &ty.kind {
    //         HirTyKind::Unit(_) => Ok(self.tctx.make_unit_ty()),

    //         HirTyKind::Path(path) => self.resolve_path(&path),
    //         HirTyKind::Ref(reference) => {
    //             let inner_ty = self.resolve_ty(&reference.ty)?;
    //             let mutability = ternary!(
    //                 reference.mutability == HirMutability::Mutable,
    //                 RefMutability::Mutable,
    //                 RefMutability::Immutable
    //             );

    //             Ok(self.tctx.make_ref_ty(inner_ty, mutability))
    //         }

    //         HirTyKind::Array(array) => {
    //             // TODO: construct the correct array ty
    //             let ty_id = self.resolve_ty(&array.ty)?;
    //             Ok(self.tctx.make_array_ty(ty_id))
    //         }

    //         HirTyKind::Slice(slice) => {
    //             // TODO: construct the correct slice ty
    //             let ty_id = self.resolve_ty(&slice.ty)?;
    //             Ok(self.tctx.make_slice_ty(ty_id))
    //         }

    //         HirTyKind::Tuple(tup) => {
    //             let mut tys = Vec::new();
    //             for el in &tup.data {
    //                 match self.resolve_ty(&el) {
    //                     Ok(ty_id) => tys.push(ty_id),
    //                     Err(diag) => {
    //                         if let Some(diag) = diag {
    //                             self.dctx.add(diag);
    //                         }

    //                         continue;
    //                     }
    //                 }
    //             }

    //             Ok(self.tctx.make_tuple_ty(tys.into()))
    //         }

    //         HirTyKind::Fn(func) => {
    //             let mut params = Vec::new();
    //             for param in &func.params {
    //                 match self.resolve_ty(&param) {
    //                     Ok(ty_id) => params.push(ty_id),
    //                     Err(diag) => {
    //                         if let Some(diag) = diag {
    //                             self.dctx.add(diag);
    //                         }

    //                         continue;
    //                     }
    //                 }
    //             }

    //             let mut ret_ty: TyId = self.tctx.make_unit_ty();
    //             if let Some(r_ty) = func.ret_ty {
    //                 match self.resolve_ty(&r_ty) {
    //                     Ok(ty_id) => ret_ty = ty_id,
    //                     Err(diag) => {
    //                         if let Some(diag) = diag {
    //                             self.dctx.add(diag);
    //                         }
    //                     }
    //                 }
    //             }

    //             Ok(self
    //                 .tctx
    //                 .make_fn_ty(Arc::new([]), None, params.into(), ret_ty))
    //         }
    //     }?;

    //     self.tctx.attach_to_hir(ty.id, Ty::new(ty_id, ty.span));
    //     Ok(ty_id)
    // }

    pub fn resolve_as_non_inferable_ty(&mut self, ty: &HirTy) -> ResolveResult<TyId> {
        let ty_id = self.resolve_ty(&ty)?;
        ternary!(
            self.tctx.is_inferred(ty_id),
            Err(Some(ResolverDiag::error(
                ty.span,
                ResolverDiagErrorKind::InvalidInference,
            ))),
            Ok(ty_id)
        )
    }
}
