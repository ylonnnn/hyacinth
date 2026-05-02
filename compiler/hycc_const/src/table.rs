use std::collections::HashMap;

use crate::constant::ConstKind;

#[derive(Debug)]
pub struct ConstTable {
    data: Vec<ConstKind>,
    table: HashMap<ConstKind, ConstId>,
}

impl ConstTable {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            table: HashMap::new(),
        }
    }

    pub fn intern(&mut self, kind: ConstKind) -> ConstId {
        if let Some(const_id) = self.table.get(&kind) {
            return *const_id;
        }

        let const_id = ConstId(self.data.len());

        self.table.insert(kind.clone(), const_id);
        self.data.push(kind);

        const_id
    }

    pub fn get(&self, const_id: ConstId) -> &ConstKind {
        &self.data[const_id.unwrap()]
    }

    pub fn get_mut(&mut self, const_id: ConstId) -> &mut ConstKind {
        &mut self.data[const_id.unwrap()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstId(usize);

impl ConstId {
    #[allow(non_upper_case_globals)]
    pub const Invalid: Self = Self(usize::MAX);

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "const id is invalid!");
        self.0
    }
}
