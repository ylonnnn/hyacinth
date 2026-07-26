use std::collections::HashMap;

use hycc_span::Span;

use crate::{
    basic_block::MirBasicBlockId,
    body::MirBody,
    decl::LocalDeclId,
    stmt::{MirStatement, MirStatementKind},
};

#[derive(Debug, Default, Clone)]
pub struct MirScopeCtx {
    pub tree: MirScopeTree,
    stack: Vec<MirScopeId>,
}

impl MirScopeCtx {
    pub fn new() -> Self {
        Self {
            tree: MirScopeTree::new(),
            stack: Vec::new(),
        }
    }

    pub fn stack(&self) -> &[MirScopeId] {
        &self.stack
    }

    pub fn push(&mut self, scope: MirScope) -> MirScopeId {
        let scope_id = self.tree.add(scope);
        self.push_id(scope_id);

        scope_id
    }

    pub fn push_id(&mut self, scope_id: MirScopeId) {
        self.stack.push(scope_id)
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn top_id(&self) -> Option<MirScopeId> {
        self.stack.last().cloned()
    }

    pub fn top(&self) -> Option<&MirScope> {
        self.top_id().map(|scope_id| self.tree.get(scope_id))
    }

    pub fn top_mut(&mut self) -> Option<&mut MirScope> {
        self.top_id().map(|scope_id| self.tree.get_mut(scope_id))
    }
}

#[derive(Debug, Default, Clone)]
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

#[derive(Debug, Clone)]
pub struct MirScope {
    // table: HashMap<LocalDeclId, (usize, MirBasicBlockId)>,
    local_decls: Vec<LocalDeclId>,

    pub(crate) children: Vec<MirScopeId>,
    pub(crate) parent: Option<MirScopeId>,

    pub exit: MirBasicBlockId,
    pub terminated: bool,
}

impl MirScope {
    pub fn new(parent: Option<MirScopeId>) -> Self {
        Self {
            // table: HashMap::new(),
            local_decls: Vec::new(),

            children: Vec::new(),
            parent,

            exit: MirBasicBlockId::Invalid,
            terminated: false,
            // term: MirScopeTerminator::Normal,
        }
    }

    pub fn local_decls(&self) -> &[LocalDeclId] {
        self.local_decls.as_slice()
        // let mut local_decls = vec![LocalDeclId::Invalid; self.table.len()];
        // self.local_decls
        //     .iter()
        //     .for_each(|(local_id, (i, _))| local_decls[*i] = *local_id);

        // local_decls.into()
    }

    pub fn add_child(&mut self, child_id: MirScopeId) {
        self.children.push(child_id)
    }

    pub fn store(&mut self, local_id: LocalDeclId) {
        self.local_decls.push(local_id)
    }

    pub fn emit_dead(&mut self, body: &mut MirBody) {
        for local_id in self.local_decls().iter().rev() {
            body.insert_stmt(MirStatement::new(
                MirStatementKind::StorageDead(*local_id),
                Span::default(),
            ));
        }
    }

    pub fn terminate(&mut self, body: &mut MirBody) {
        if self.terminated {
            return;
        }

        self.emit_dead(body);
        self.terminated = true
    }

    // pub fn store(&mut self, local_id: LocalDeclId, block_id: MirBasicBlockId) {
    //     self.table.insert(local_id, (self.table.len(), block_id));
    // }

    // pub fn drop(&mut self, local_id: LocalDeclId) {
    //     self.table.remove(&local_id);
    // }

    // pub fn local_init_block(&self, local_id: LocalDeclId) -> Option<MirBasicBlockId> {
    //     self.table.get(&local_id).map(|(_, block_id)| *block_id)
    // }
}
