use std::collections::HashMap;

use hycc_span::Span;
use hycc_symbol::Symbol;

use crate::{HirId, item::HirItemAccessibility};

#[derive(Debug)]
pub struct DefinitionTable {
    data: Vec<Definition>,
    map: HashMap<HirId, DefId>,
}

impl DefinitionTable {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, definition: Definition) -> DefId {
        self.data.push(definition);
        DefId(self.data.len() - 1)
    }

    pub fn get(&self, id: DefId) -> &Definition {
        &self.data[id.unwrap()]
    }

    pub fn define_hir(&mut self, hir_id: HirId, definition: Definition) -> DefId {
        let def_id = self.insert(definition);
        self.map.insert(hir_id, def_id);

        def_id
    }

    pub fn get_def_id(&self, hir_id: HirId) -> Option<&DefId> {
        self.map.get(&hir_id)
    }

    pub fn get_def(&self, hir_id: HirId) -> Option<&Definition> {
        self.get_def_id(hir_id).map(|def_id| self.get(*def_id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefId(usize);

impl DefId {
    #[allow(non_upper_case_globals)]
    pub const Invalid: Self = Self(usize::MAX);

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "definition id is not valid!");
        self.0
    }
}

#[derive(Debug, Clone)]
pub enum DefKind {
    Petal,

    Fn,
    Var,
}

impl DefKind {
    pub fn space(&self) -> DefSpace {
        match self {
            Self::Petal => DefSpace::Type,

            Self::Fn => DefSpace::Value,
            Self::Var => DefSpace::Value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DefSpace {
    Type,
    Value,
}

pub type DefAccessibility = HirItemAccessibility;

#[derive(Debug, Clone)]
pub struct Definition {
    pub name: Symbol,
    pub kind: DefKind,
    pub hir_id: HirId,
    pub span: Span,
    pub accessibility: DefAccessibility,
}

impl Definition {
    pub fn new(
        name: Symbol,
        kind: DefKind,
        hir_id: HirId,
        span: Span,
        accessibility: DefAccessibility,
    ) -> Self {
        Self {
            name,
            kind,
            hir_id,
            span,
            accessibility,
        }
    }

    pub fn new_default(name: Symbol, kind: DefKind, hir_id: HirId, span: Span) -> Self {
        Self::new(name, kind, hir_id, span, DefAccessibility::Priv)
    }
}
