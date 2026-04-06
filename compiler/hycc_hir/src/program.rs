use crate::{HirId, item::HirItem};

#[derive(Debug, Clone)]
pub struct HirProgram<'h> {
    pub id: HirId,
    pub items: Vec<&'h HirItem<'h>>,
}

impl<'h> HirProgram<'h> {
    pub fn new(items: Vec<&'h HirItem<'h>>) -> Self {
        Self {
            id: HirId::Invalid,
            items,
        }
    }
}
