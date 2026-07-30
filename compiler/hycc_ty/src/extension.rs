use std::collections::{HashMap, hash_map::Entry};

use hycc_hir::def::{Binding, DefId, DefSpace};
use hycc_symbol::Symbol;

use crate::context::TyId;

#[derive(Debug, Clone)]
pub struct ExtensionTable {
    data: Vec<Extension>,
    native: HashMap<DefId, Vec<ExtensionId>>,
    // TODO: protocol: HashMap<DefId, Vec<ExtensionId>>
}

impl ExtensionTable {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            native: HashMap::new(),
        }
    }

    pub fn insert(&mut self, ext: Extension) -> ExtensionId {
        self.data.push(ext);
        ExtensionId(self.data.len() - 1)
    }

    pub fn get(&self, id: ExtensionId) -> &Extension {
        &self.data[id.unwrap()]
    }

    pub fn get_mut(&mut self, id: ExtensionId) -> &mut Extension {
        &mut self.data[id.unwrap()]
    }

    pub fn attach(&mut self, def_id: DefId, ext: Extension) -> ExtensionId {
        let ext_id = self.insert(ext);
        self.attach_id(def_id, ext_id);

        ext_id
    }

    pub fn attach_id(&mut self, def_id: DefId, ext_id: ExtensionId) {
        match self.native.entry(def_id) {
            Entry::Vacant(entry) => {
                entry.insert(vec![ext_id]);
            }

            Entry::Occupied(mut entry) => {
                let extensions: &mut Vec<ExtensionId> = entry.get_mut();
                extensions.push(ext_id);
            }
        }
    }

    pub fn get_def_native_exts(&self, def_id: DefId) -> Option<&[ExtensionId]> {
        self.native.get(&def_id).map(|exts| exts.as_slice())
    }
}

#[derive(Debug, Clone)]
pub struct Extension {
    pub target: TyId,
    items: HashMap<(DefSpace, Symbol), Binding>,
}

impl Extension {
    pub fn new(target: TyId, items: HashMap<(DefSpace, Symbol), Binding>) -> Self {
        Self { target, items }
    }

    pub fn get(&self, space: DefSpace, name: Symbol) -> Option<&Binding> {
        self.items.get(&(space, name))
    }

    pub fn get_mut(&mut self, space: DefSpace, name: Symbol) -> Option<&mut Binding> {
        self.items.get_mut(&(space, name))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExtensionId(usize);

impl ExtensionId {
    #[allow(non_upper_case_globals)]
    pub const Invalid: Self = Self(usize::MAX);

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "extension id is not valid!");
        self.0
    }
}
