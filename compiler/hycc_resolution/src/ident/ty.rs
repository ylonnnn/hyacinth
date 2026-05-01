use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    def::DefSpace,
    ty::{HirTy, HirTyKind},
};

use crate::{ResolveResult, ident::resolver::Resolver};

impl<'c> Resolver<'c> {
    pub(crate) fn resolve_ty(&mut self, ty: &HirTy) -> ResolveResult {
        self.expect_space(DefSpace::Type, |s| match &ty.kind {
            HirTyKind::Unit(..) => Ok(()),

            HirTyKind::Path(path) => s.resolve_path(path),
            HirTyKind::Ref(reference) => s.resolve_ty(&reference.ty),

            HirTyKind::Array(array) => {
                if let Err(Some(diag)) = s.resolve_expr(&array.size) {
                    s.dctx.add(diag);
                }

                s.resolve_ty(&array.ty)
            }

            HirTyKind::Slice(slice) => s.resolve_ty(&slice.ty),

            HirTyKind::Tuple(tup) => {
                for element in &tup.data {
                    if let Err(Some(diag)) = s.resolve_ty(&element) {
                        s.dctx.add(diag);
                    }
                }

                Ok(())
            }

            HirTyKind::Fn(func) => {
                for param in &func.params {
                    if let Err(Some(diag)) = s.resolve_ty(&param) {
                        s.dctx.add(diag);
                    }
                }

                if let Some(ret_ty) = func.ret_ty {
                    if let Err(Some(diag)) = s.resolve_ty(&ret_ty) {
                        s.dctx.add(diag);
                    }
                }

                Ok(())
            }
        })
    }
}
