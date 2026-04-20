use hycc_hir::item::HirPetal;

#[derive(Debug)]
pub struct TyInferer {}

impl TyInferer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn infer(&mut self, tree: &HirPetal) {
        for item in &tree.items {
            self.infer_item(&item);
        }
    }
}
