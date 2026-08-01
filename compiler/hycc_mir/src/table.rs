use std::collections::HashMap;

use hycc_hir::def::DefId;
use hycc_util::bug;

use crate::body::{MirBody, MirBodyId};

#[derive(Debug)]
pub struct MirTable {
    data: Vec<MirBody>,
    defs: HashMap<DefId, MirBodyId>,
    stack: Vec<MirBodyId>,
}

impl MirTable {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            defs: HashMap::new(),
            stack: Vec::new(),
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

    pub fn get(&self, id: MirBodyId) -> &MirBody {
        &self.data[id.unwrap()]
    }

    pub fn get_mut(&mut self, id: MirBodyId) -> &mut MirBody {
        &mut self.data[id.unwrap()]
    }

    pub fn get_id_by_def(&self, def_id: DefId) -> Option<MirBodyId> {
        self.defs.get(&def_id).cloned()
    }

    pub fn get_by_def(&self, def_id: DefId) -> &MirBody {
        let Some(id) = self.get_id_by_def(def_id) else {
            bug!("def id {def_id:?} has no attached body")
        };

        self.get(id)
    }

    pub fn get_mut_by_def(&mut self, def_id: DefId) -> &mut MirBody {
        let Some(id) = self.get_id_by_def(def_id) else {
            bug!("def id {def_id:?} has no attached body")
        };

        self.get_mut(id)
    }

    pub fn push_new(&mut self) -> MirBodyId {
        self.push(MirBody::new())
    }

    pub fn push_new_for(&mut self, def_id: DefId) -> MirBodyId {
        let body_id = self.new_body_for(def_id);
        self.push_id(body_id);

        body_id
    }

    pub fn push(&mut self, body: MirBody) -> MirBodyId {
        let body_id = self.insert_body(body);
        self.push_id(body_id);

        body_id
    }

    pub fn push_id(&mut self, body_id: MirBodyId) {
        self.stack.push(body_id);
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn top_id(&self) -> Option<MirBodyId> {
        self.stack.last().cloned()
    }

    pub fn top(&self) -> Option<&MirBody> {
        self.top_id().map(|body_id| self.get(body_id))
    }

    pub fn top_mut(&mut self) -> Option<&mut MirBody> {
        self.top_id().map(|body_id| self.get_mut(body_id))
    }
}
