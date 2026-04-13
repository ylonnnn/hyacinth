use hycc_hir::item::HirPetal;

use crate::diag::ResolverDiag;

#[derive(Debug)]
pub struct Resolver {}

pub type ResolveResult<T = (), E = ResolverDiag> = Result<T, E>;

impl Resolver {
    pub fn new() -> Self {
        Self {}
    }

    pub fn resolve(&mut self, tree: &HirPetal) -> ResolveResult {
        todo!()
    }
}
