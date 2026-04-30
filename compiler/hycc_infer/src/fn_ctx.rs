use hycc_ty::ty::Ty;

#[derive(Debug)]
pub struct FnCtx {
    pub ty: Ty,
}

impl FnCtx {
    pub fn new(ty: Ty) -> Self {
        Self { ty }
    }
}
