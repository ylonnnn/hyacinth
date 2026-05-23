use std::collections::HashMap;

use crate::{basic_block::MirBasicBlockId, local::LocalDeclId};

#[derive(Debug, Default)]
pub struct MirScopeTree(Vec<MirScope>);

impl MirScopeTree {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, scope: MirScope) -> MirScopeId {
        self.0.push(scope);
        MirScopeId(self.0.len() - 1)
    }

    pub fn get(&self, scope_id: MirScopeId) -> &MirScope {
        &self.0[scope_id.unwrap()]
    }

    pub fn get_mut(&mut self, scope_id: MirScopeId) -> &mut MirScope {
        &mut self.0[scope_id.unwrap()]
    }

    pub fn create(&mut self, scope_id: Option<MirScopeId>) -> MirScopeId {
        let child_id = self.add(MirScope::new(scope_id));

        if let Some(scope_id) = scope_id {
            let parent = self.get_mut(scope_id);
            parent.add_child(child_id);
        }

        child_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MirScopeId(pub(crate) usize);

impl MirScopeId {
    #[allow(non_upper_case_globals)]
    pub const Invalid: Self = Self(usize::MAX);

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "mir scope id is invalid!");
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MirScopeTerminator {
    Normal,
    Ret,
}

#[derive(Debug)]
pub struct MirScope {
    table: HashMap<LocalDeclId, (usize, MirBasicBlockId)>,

    pub(crate) children: Vec<MirScopeId>,
    pub(crate) parent: Option<MirScopeId>,

    pub exit: MirBasicBlockId,
    pub term: MirScopeTerminator,
}

impl MirScope {
    pub fn new(parent: Option<MirScopeId>) -> Self {
        Self {
            table: HashMap::new(),
            parent,
            children: Vec::new(),
            exit: MirBasicBlockId::Invalid,
            term: MirScopeTerminator::Normal,
        }
    }

    pub fn local_decls(&self) -> Box<[LocalDeclId]> {
        let mut local_decls = vec![LocalDeclId::Invalid; self.table.len()];
        self.table
            .iter()
            .for_each(|(local_id, (i, _))| local_decls[*i] = *local_id);

        local_decls.into()
    }

    pub fn add_child(&mut self, child_id: MirScopeId) {
        self.children.push(child_id)
    }

    pub fn store(&mut self, local_id: LocalDeclId, block_id: MirBasicBlockId) {
        self.table.insert(local_id, (self.table.len(), block_id));
    }

    pub fn drop(&mut self, local_id: LocalDeclId) {
        self.table.remove(&local_id);
    }

    pub fn local_init_block(&self, local_id: LocalDeclId) -> Option<MirBasicBlockId> {
        self.table.get(&local_id).map(|(_, block_id)| *block_id)
    }
}
