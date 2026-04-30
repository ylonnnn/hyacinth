use hycc_hir::HirId;
use hycc_ty::ty::Ty;

#[derive(Debug)]
pub struct FnCtx {
    pub ty: Ty,
    pub block: HirId,
}

impl FnCtx {
    pub fn new(ty: Ty, block: HirId) -> Self {
        Self { ty, block }
    }
}
