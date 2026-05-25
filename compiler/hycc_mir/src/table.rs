use std::collections::HashMap;

use hycc_hir::def::DefId;
use hycc_util::bug;

use crate::body::{MirBody, MirBodyId};

#[derive(Debug)]
pub struct MirTable {
    data: Vec<MirBody>,
    defs: HashMap<DefId, MirBodyId>,
}

impl MirTable {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            defs: HashMap::new(),
        }
    }

    pub fn bodies(&self) -> &[MirBody] {
        &self.data
    }

    pub fn defs(&self) -> &HashMap<DefId, MirBodyId> {
        &self.defs
    }

    pub fn new_body(&mut self) -> MirBodyId {
        self.insert_body(MirBody::new())
    }

    pub fn new_body_for(&mut self, def_id: DefId) -> MirBodyId {
        self.insert_body_for(def_id, MirBody::new())
    }

    pub fn insert_body(&mut self, body: MirBody) -> MirBodyId {
        self.data.push(body);
        MirBodyId(self.data.len() - 1)
    }

    pub fn insert_body_for(&mut self, def_id: DefId, body: MirBody) -> MirBodyId {
        if self.defs.contains_key(&def_id) {
            bug!("def id {def_id:?} already has an attached body!")
        }

        let body_id = self.insert_body(body);
        self.defs.insert(def_id, body_id);

        body_id
    }

    pub fn get_body(&self, id: MirBodyId) -> &MirBody {
        &self.data[id.unwrap()]
    }

    pub fn get_body_mut(&mut self, id: MirBodyId) -> &mut MirBody {
        &mut self.data[id.unwrap()]
    }

    pub fn get_by_def(&self, def_id: DefId) -> &MirBody {
        match self.defs.get(&def_id) {
            Some(id) => self.get_body(*id),
            _ => bug!("def id {def_id:?} has no attached body"),
        }
    }

    pub fn get_mut_by_def(&mut self, def_id: DefId) -> &mut MirBody {
        match self.defs.get(&def_id) {
            Some(id) => self.get_body_mut(*id),
            _ => bug!("def id {def_id:?} has no attached body"),
        }
    }
}
