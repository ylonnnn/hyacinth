use hycc_hir::HirId;
use hycc_ty::ty::Ty;

#[derive(Debug, Clone)]
pub struct FnCtx {
    pub ty: Ty,
    pub fn_body: HirId,
}

impl FnCtx {
    pub fn new(ty: Ty, fn_body: HirId) -> Self {
        Self { ty, fn_body }
    }
}
