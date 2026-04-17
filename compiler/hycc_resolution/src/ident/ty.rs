use hycc_diagnostic::DiagnosticContext;
use hycc_hir::{
    def::DefSpace,
    ty::{HirTy, HirTyKind},
};

use crate::ident::resolver::{ResolveResult, Resolver};

impl<'s, 'd> Resolver<'s, 'd> {
    pub(crate) fn resolve_ty(&mut self, ty: &HirTy) -> ResolveResult {
        self.expect_space(DefSpace::Type, |s| match &ty.kind {
            HirTyKind::Path(path) => s.resolve_path(path),
            HirTyKind::Unit(..) => Ok(()),
            HirTyKind::Array(array) => {
                if let Err(Some(diag)) = s.resolve_expr(&array.size) {
                    s.dctx.add(diag);
                }

                s.resolve_ty(&array.ty)
            }
        })
    }
}
