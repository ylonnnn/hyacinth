use hycc_hir::HirId;
use hycc_ty::ty::Ty;

#[derive(Debug)]
pub struct FnCtx {
    pub ty: Ty,
    pub fn_body: HirId,
    // TODO: separate ty into params and ret_ty for better diagnostics
}

impl FnCtx {
    pub fn new(ty: Ty, fn_body: HirId) -> Self {
        Self { ty, fn_body }
    }
}
