use crate::{HirId, item::HirItem};

#[derive(Debug, Clone)]
pub struct HirProgram {
    pub id: HirId,
    pub items: Vec<HirItem>,
}
