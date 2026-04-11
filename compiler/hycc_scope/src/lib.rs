use std::collections::HashMap;

use hycc_hir::{
    HirId,
    def::{DefId, DefSpace},
};
use hycc_symbol::Symbol;
use hycc_util::ternary;

#[derive(Debug, Clone)]
pub struct ScopeTable {
    data: Vec<Scope>,
}

impl ScopeTable {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn insert(&mut self, scope: Scope) -> ScopeId {
        self.data.push(scope);
        ScopeId(self.data.len() - 1)
    }

    pub fn get(&self, id: ScopeId) -> &Scope {
        &self.data[id.unwrap()]
    }

    pub fn get_mut(&mut self, id: ScopeId) -> &mut Scope {
        &mut self.data[id.unwrap()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(usize);

impl ScopeId {
    #[allow(non_snake_case)]
    pub fn Invalid() -> Self {
        Self(usize::MAX)
    }

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "scope id is not valid");
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct ScopeCtx {
    table: ScopeTable,
    stack: Vec<ScopeId>,
    node_table: HashMap<HirId, ScopeId>,
    def_table: HashMap<DefId, ScopeId>,
}

impl ScopeCtx {
    pub fn new() -> Self {
        let mut inst = Self {
            table: ScopeTable::new(),
            stack: Vec::new(),
            node_table: HashMap::new(),
            def_table: HashMap::new(),
        };

        inst.stack.push(inst.table.insert(Scope::new()));
        inst
    }

    pub fn attach(&mut self, hir_id: HirId, scope: Scope) -> ScopeId {
        let scope_id = self.table.insert(scope);
        self.node_table.insert(hir_id, scope_id);

        scope_id
    }

    pub fn attach_id(&mut self, hir_id: HirId, scope_id: ScopeId) {
        self.node_table.insert(hir_id, scope_id);
    }

    pub fn attach_to_def(&mut self, def_id: DefId, scope: Scope) -> ScopeId {
        let scope_id = self.table.insert(scope);
        self.def_table.insert(def_id, scope_id);

        scope_id
    }

    pub fn attach_id_to_def(&mut self, def_id: DefId, scope_id: ScopeId) {
        self.def_table.insert(def_id, scope_id);
    }

    pub fn push(&mut self, scope: Scope) -> ScopeId {
        let id = self.table.insert(scope);
        self.stack.push(id);

        id
    }

    pub fn push_id(&mut self, scope_id: ScopeId) {
        self.stack.push(scope_id)
    }

    pub fn pop(&mut self) -> bool {
        ternary!(self.stack.len() <= 1, false, {
            self.stack.pop();
            true
        })
    }

    pub fn top(&self) -> &Scope {
        self.table.get(*self.stack.last().unwrap())
    }

    pub fn top_mut(&mut self) -> &mut Scope {
        self.table.get_mut(*self.stack.last_mut().unwrap())
    }

    pub fn enter<F>(&mut self, scope: Scope, mut handler: F)
    where
        F: FnMut(&mut Self),
    {
        self.push(scope);
        handler(self);
        self.pop();
    }

    pub fn enter_by_id<F>(&mut self, scope_id: ScopeId, mut handler: F)
    where
        F: FnMut(&mut Self),
    {
        self.push_id(scope_id);
        handler(self);
        self.pop();
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    definitions: HashMap<DefSpace, HashMap<Symbol, DefId>>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
        }
    }

    pub fn define(&mut self, space: DefSpace, name: Symbol, def_id: DefId) {
        use std::collections::hash_map::Entry;

        match self.definitions.entry(space) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().insert(name, def_id);
            }

            Entry::Vacant(entry) => {
                entry.insert(HashMap::new());
                self.define(space, name, def_id);
            }
        }
    }

    pub fn get(&self, space: DefSpace, name: Symbol) -> Option<DefId> {
        let Some(defs) = self.definitions.get(&space) else {
            return None;
        };

        Some(*defs.get(&name)?)
    }
}
