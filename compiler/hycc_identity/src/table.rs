use std::collections::HashMap;

use hycc_symbol::Symbol;

use crate::identity::Identity;

#[derive(Debug)]
pub struct IdentityTable {
    data: Vec<Identity>,
    ids: HashMap<Symbol, IdentityId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentityId {
    Valid(usize),
    Invalid,
}

impl IdentityId {
    pub fn unwrap(&self) -> usize {
        match self {
            Self::Valid(id) => *id,
            _ => panic!("identity id is invalid"),
        }
    }
}

impl IdentityTable {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            ids: HashMap::new(),
        }
    }

    pub fn define(&mut self, identity: Identity) -> IdentityId {
        use std::collections::hash_map::Entry;

        match self.ids.entry(identity.name) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let id = IdentityId::Valid(self.data.len());

                self.data.push(identity);
                *entry.insert(id)
            }
        }
    }

    pub fn get(&self, id: IdentityId) -> &Identity {
        &self.data[id.unwrap()]
    }
}
