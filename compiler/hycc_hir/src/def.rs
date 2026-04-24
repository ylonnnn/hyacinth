use std::{collections::HashMap, fmt::Display};

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

    pub fn defs(&self) -> &[Definition] {
        &self.data
    }

    pub fn insert(&mut self, definition: Definition) -> DefId {
        self.data.push(definition);
        DefId(self.data.len() - 1)
    }

    pub fn get(&self, id: DefId) -> &Definition {
        &self.data[id.unwrap()]
    }

    pub fn get_mut(&mut self, id: DefId) -> &mut Definition {
        &mut self.data[id.unwrap()]
    }

    pub fn define_hir(&mut self, hir_id: HirId, definition: Definition) -> DefId {
        let def_id = self.insert(definition);
        self.map.insert(hir_id, def_id);

        def_id
    }

    pub fn define_id_hir(&mut self, hir_id: HirId, def_id: DefId) {
        self.map.insert(hir_id, def_id);
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
    Builtin(BuiltinKind),

    Petal,

    Fn(Box<FnDef>),
    FnParam,

    Struct(Box<StructDef>),

    Var,
}

impl DefKind {
    pub fn article(&self) -> &'static str {
        match self {
            Self::Builtin(_)
            | Self::Petal
            | Self::Fn(_)
            | Self::FnParam
            | Self::Struct(_)
            | Self::Var => "a",
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::Builtin(_) => "built-in",
            Self::Petal => "petal",

            Self::Fn(_) => "function",
            Self::FnParam => "function parameter",

            Self::Struct(_) => "struct",

            Self::Var => "variable",
        }
    }

    pub fn space(&self) -> DefSpace {
        if let Self::Builtin(kind) = self {
            return match &kind {
                BuiltinKind::Ty(_) => DefSpace::Type,
            };
        }

        match self {
            Self::Builtin(_) => unreachable!(),

            Self::Petal | Self::Struct(_) => DefSpace::Type,

            Self::Fn(_) | Self::FnParam | Self::Var => DefSpace::Value,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BuiltinKind {
    Ty(BuiltinTyKind),
}

#[derive(Debug, Clone)]
pub enum BuiltinTyKind {
    Unit,

    Int(BuiltinIntTy),
    Float(u8),

    Bool,

    Char,
    String,

    Infer,
}

#[derive(Debug, Clone)]
pub enum BuiltinIntTy {
    Fixed(u8, bool),
    Size(bool),
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub fields: Vec<StructFieldDef>,
    pub field_map: HashMap<Symbol, usize>,
}

impl StructDef {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            field_map: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StructFieldDef {
    pub name: Symbol,
    pub span: Span,
    pub accessibility: DefAccessibility,
    pub ty: HirId,
}

#[derive(Debug, Clone)]
pub struct FnDef {
    pub params: Vec<DefId>,
    pub ret_ty: Option<HirId>,
}

impl FnDef {
    pub fn new(ret_ty: Option<HirId>) -> Self {
        Self {
            params: Vec::new(),
            ret_ty,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DefSpace {
    Type,
    Value,
}

impl Display for DefSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Type => "type",
                Self::Value => "value",
            }
        )
    }
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

    pub fn builtin(name: Symbol, kind: BuiltinKind, accessibility: DefAccessibility) -> Self {
        Self::new(
            name,
            DefKind::Builtin(kind),
            HirId::Invalid,
            Span::default(),
            accessibility,
        )
    }

    pub fn new_default(name: Symbol, kind: DefKind, hir_id: HirId, span: Span) -> Self {
        Self::new(name, kind, hir_id, span, DefAccessibility::Priv)
    }
}
