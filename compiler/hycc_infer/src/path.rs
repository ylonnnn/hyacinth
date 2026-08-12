use hycc_hir::path::HirPath;
use hycc_resolution::resolver_traits::InstantiateIdent;
use hycc_ty::context::TyId;

use crate::{
    diag::InferDiag,
    inferer::{InferResult, TyInferer},
};

impl<'t, 'd, 'c, 'h, 'p> TyInferer<'t, 'd, 'c, 'h, 'p> {
    pub(crate) fn infer_path(&mut self, path: &HirPath) -> InferResult<TyId> {
        let ty_id = self.tctx.expect_hir_ty_id(path.id);
        Ok(self.tctx.resolve_ty(ty_id))
    }
}
