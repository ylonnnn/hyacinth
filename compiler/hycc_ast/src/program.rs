use crate::Item;

#[derive(Debug)]
pub struct Program {
    pub items: Vec<Item>,
}

impl Program {
    pub fn new(items: Vec<Item>) -> Self {
        Self { items }
    }
}
