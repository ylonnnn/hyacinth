use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    def::DefSpace,
    ty::{HirTy, HirTyKind},
};

use crate::{ResolveResult, ident::resolver::Resolver};

impl<'s, 'd> Resolver<'s, 'd> {
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
        })
    }
}
