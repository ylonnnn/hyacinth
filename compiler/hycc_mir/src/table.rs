use std::collections::HashMap;

use hycc_hir::def::DefId;
use hycc_util::bug;

use crate::body::MirBody;

#[derive(Debug)]
pub struct MirTable(HashMap<DefId, MirBody>);

impl MirTable {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn defs(&self) -> &HashMap<DefId, MirBody> {
        &self.0
    }

    pub fn insert(&mut self, def_id: DefId, body: MirBody) -> &mut MirBody {
        if self.0.contains_key(&def_id) {
            bug!("def id {def_id:?} already has an attached body!")
        }

        self.0.insert(def_id, body);
        self.0.get_mut(&def_id).unwrap()
    }

    pub fn get(&self, def_id: DefId) -> &MirBody {
        match self.0.get(&def_id) {
            Some(body) => body,
            _ => bug!("def id {def_id:?} has no attached body"),
        }
    }

    pub fn get_mut(&mut self, def_id: DefId) -> &mut MirBody {
        match self.0.get_mut(&def_id) {
            Some(body) => body,
            _ => bug!("def id {def_id:?} has no attached body"),
        }
    }
}
