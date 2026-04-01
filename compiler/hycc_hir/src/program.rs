use crate::item::HirItem;

#[derive(Debug, Clone)]
pub struct HirProgram {
    pub items: Vec<HirItem>,
}
