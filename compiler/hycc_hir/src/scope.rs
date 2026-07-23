use std::collections::HashMap;

use crate::{
    HirId,
    def::{Binding, DefId, DefSpace},
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
    pub stack: Vec<ScopeId>, // TEMP
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

    pub fn scopes(&self) -> &[Scope] {
        &self.table.data
    }

    pub fn root_id(&self) -> ScopeId {
        ScopeId(0)
    }

    pub fn attach(&mut self, hir_id: HirId, scope: Scope) -> ScopeId {
        assert!(
            !self.node_table.contains_key(&hir_id),
            "hir node already has a scope attached!"
        );

        let scope_id = self.table.insert(scope);
        self.attach_id(hir_id, scope_id);

        scope_id
    }

    pub fn try_attach(&mut self, hir_id: HirId, scope: Scope) -> ScopeId {
        if let Some(scope_id) = self.node_table.get(&hir_id) {
            *scope_id
        } else {
            self.attach(hir_id, scope)
        }
    }

    pub fn attach_id(&mut self, hir_id: HirId, scope_id: ScopeId) {
        assert!(
            !self.node_table.contains_key(&hir_id),
            "hir node already has a scope attached!"
        );

        self.node_table.insert(hir_id, scope_id);
    }

    pub fn try_attach_id(&mut self, hir_id: HirId, scope_id: ScopeId) -> ScopeId {
        if let Some(scope_id) = self.node_table.get(&hir_id) {
            *scope_id
        } else {
            self.attach_id(hir_id, scope_id);
            scope_id
        }
    }

    pub fn get(&self, id: ScopeId) -> &Scope {
        self.table.get(id)
    }

    pub fn get_mut(&mut self, id: ScopeId) -> &mut Scope {
        self.table.get_mut(id)
    }

    pub fn get_id_by_hir(&self, hir_id: HirId) -> Option<ScopeId> {
        Some(*self.node_table.get(&hir_id)?)
    }

    pub fn get_by_hir(&self, hir_id: HirId) -> Option<&Scope> {
        let scope_id = self.get_id_by_hir(hir_id)?;
        Some(self.table.get(scope_id))
    }

    pub fn get_mut_by_hir(&mut self, hir_id: HirId) -> Option<&mut Scope> {
        let scope_id = self.get_id_by_hir(hir_id)?;
        Some(self.table.get_mut(scope_id))
    }

    pub fn attach_to_def(&mut self, def_id: DefId, scope: Scope) -> ScopeId {
        assert!(
            !self.def_table.contains_key(&def_id),
            "definition already has a scope attached!"
        );

        let scope_id = self.table.insert(scope);
        self.attach_id_to_def(def_id, scope_id);

        scope_id
    }

    pub fn try_attach_to_def(&mut self, def_id: DefId, scope: Scope) -> ScopeId {
        if let Some(scope_id) = self.def_table.get(&def_id) {
            *scope_id
        } else {
            self.attach_to_def(def_id, scope)
        }
    }

    pub fn attach_id_to_def(&mut self, def_id: DefId, scope_id: ScopeId) {
        assert!(
            !self.def_table.contains_key(&def_id),
            "definition already has a scope attached!"
        );

        self.def_table.insert(def_id, scope_id);
    }

    pub fn try_attach_id_to_def(&mut self, def_id: DefId, scope_id: ScopeId) -> ScopeId {
        if let Some(scope_id) = self.def_table.get(&def_id) {
            *scope_id
        } else {
            self.attach_id_to_def(def_id, scope_id);
            scope_id
        }
    }

    pub fn get_id_from_def(&self, def_id: DefId) -> Option<ScopeId> {
        Some(*self.def_table.get(&def_id)?)
    }

    pub fn get_from_def(&self, def_id: DefId) -> Option<&Scope> {
        let scope_id = self.get_id_from_def(def_id)?;
        Some(self.table.get(scope_id))
    }

    pub fn get_mut_from_def(&mut self, def_id: DefId) -> Option<&mut Scope> {
        let scope_id = self.get_id_from_def(def_id)?;
        Some(self.table.get_mut(scope_id))
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

    pub fn top_id(&self) -> ScopeId {
        self.stack.last().cloned().unwrap()
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

    pub fn get_def<F>(
        &self,
        space: Option<DefSpace>,
        name: Symbol,
        mut stop_cond: F,
    ) -> Option<(&Binding, ScopeId)>
    where
        F: FnMut(&Self, ScopeId, usize) -> bool,
    {
        let mut depth = 0;
        for scope_id in self.stack.iter().rev() {
            let stop = stop_cond(self, *scope_id, depth);

            let binding = self.table.get(*scope_id).get(space, name);
            if binding.is_some() {
                return binding.map(|binding| (binding, *scope_id));
            }

            depth += 1;

            if stop {
                break;
            }
        }

        None
    }

    pub fn get_def_current_only(
        &self,
        space: Option<DefSpace>,
        name: Symbol,
    ) -> Option<(&Binding, ScopeId)> {
        self.get_def(space, name, |_, _, _| true)
    }

    pub fn get_def_until_scope(
        &self,
        space: Option<DefSpace>,
        name: Symbol,
        scope_id: ScopeId,
    ) -> Option<&Binding> {
        self.get_def(space, name, |_, s_id, _| s_id == scope_id)
            .map(|(binding, _)| binding)
    }

    pub fn get_def_until_root(
        &self,
        space: Option<DefSpace>,
        name: Symbol,
    ) -> Option<(&Binding, ScopeId)> {
        self.get_def(space, name, |_, _, _| false)
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    definitions: HashMap<(DefSpace, Symbol), Binding>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
        }
    }

    pub fn define(&mut self, space: DefSpace, name: Symbol, binding: Binding) -> &mut Binding {
        self.definitions.insert((space, name), binding);
        self.definitions.get_mut(&(space, name)).unwrap()
    }

    pub fn get(&self, space: Option<DefSpace>, name: Symbol) -> Option<&Binding> {
        match space {
            Some(space) => self.definitions.get(&(space, name)),
            None => [DefSpace::Type, DefSpace::Value]
                .into_iter()
                .map(|space| self.definitions.get(&(space, name)))
                .find(|def| def.is_some())
                .unwrap(),
        }
    }
}
