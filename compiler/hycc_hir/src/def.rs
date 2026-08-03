use std::{collections::HashMap, fmt::Display};

use hycc_span::Span;
use hycc_symbol::Symbol;

use crate::{
    HirId, HirMutability,
    generic::HirGenericParamKind,
    item::{HirItemAccessibility, HirItemLevel, HirPubAccessibilityKind},
    petal::PetalId,
};

#[derive(Debug)]
pub struct DefinitionTable {
    data: Vec<Definition>,
    map: HashMap<HirId, DefId>,
    pub builtins: Vec<DefId>,
}

impl DefinitionTable {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            map: HashMap::new(),
            builtins: Vec::new(),
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

    pub fn get_def_id(&self, hir_id: HirId) -> Option<DefId> {
        self.map.get(&hir_id).map(|def_id| *def_id)
    }

    pub fn get_def(&self, hir_id: HirId) -> Option<&Definition> {
        self.get_def_id(hir_id).map(|def_id| self.get(def_id))
    }

    pub fn expect_def_id(&self, hir_id: HirId) -> DefId {
        self.get_def_id(hir_id).expect(&format!(
            "expected a def id attached to hir id {:?}",
            hir_id
        ))
    }

    pub fn expect_def(&self, hir_id: HirId) -> &Definition {
        self.get(self.expect_def_id(hir_id))
    }

    pub fn expect_mut_def(&mut self, hir_id: HirId) -> &mut Definition {
        self.get_mut(self.expect_def_id(hir_id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefId(usize);

impl DefId {
    pub fn new(data: usize) -> Self {
        Self(data)
    }

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
    Proto,

    Adt(AdtKind),

    GenericParam(Box<GenericParamDef>),

    Fn(Box<FnDef>),
    FnParam,

    Var(Box<VarDef>),
}

impl DefKind {
    pub fn article(&self) -> &'static str {
        match self {
            Self::Builtin(_)
            | Self::Petal
            | Self::Proto
            | Self::GenericParam(_)
            | Self::Fn(_)
            | Self::FnParam
            | Self::Var(_) => "a",

            Self::Adt(kind) => kind.article(),
        }
    }

    pub fn kind(&self) -> String {
        match self {
            Self::Builtin(_) => String::from("built-in"),

            Self::Petal => String::from("petal"),
            Self::Proto => String::from("protocol"),

            Self::Adt(kind) => kind.kind().into(),

            Self::GenericParam(_) => String::from("type parameter"),

            Self::Fn(_) => String::from("function"),
            Self::FnParam => String::from("function parameter"),

            Self::Var(var_def) => format!(
                "{}variable",
                match &var_def.level {
                    HirItemLevel::Top => "top-level ",
                    _ => "",
                }
            ),
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

            Self::Petal | Self::Proto | Self::Adt(_) | Self::GenericParam(_) => DefSpace::Type,

            Self::Fn(_) | Self::FnParam | Self::Var(_) => DefSpace::Value,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BuiltinKind {
    Ty(BuiltinTyKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltinTyKind {
    Unit,

    Int(BuiltinIntTy),
    Float(u8),

    Bool,

    Char,
    String,

    Infer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltinIntTy {
    Fixed(u8, bool),
    Size(bool),
}

#[derive(Debug, Clone)]
pub struct ProtoDef {}

#[derive(Debug, Clone)]
pub enum ProtoDefItem {}

#[derive(Debug, Clone)]
pub enum AdtKind {
    Struct(Box<StructDef>),
    // TODO: Enum
}

impl AdtKind {
    pub fn article(&self) -> &'static str {
        match &self {
            Self::Struct(_) => "a",
        }
    }

    pub fn kind(&self) -> &'static str {
        match &self {
            Self::Struct(_) => "struct",
        }
    }
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
pub struct GenericParamDef {
    pub idx: usize,
    pub kind: HirGenericParamKind,
}

#[derive(Debug, Clone)]
pub struct FnDef {
    pub generic_params: Vec<DefId>,
    pub params: Vec<DefId>,
    pub ret_ty: Option<HirId>,
}

impl FnDef {
    pub fn new(ret_ty: Option<HirId>) -> Self {
        Self {
            generic_params: Vec::new(),
            params: Vec::new(),
            ret_ty,
        }
    }
}

pub type DefMutability = HirMutability;

#[derive(Debug, Clone)]
pub struct VarDef {
    pub level: HirItemLevel,
    pub mutability: DefMutability,
}

impl VarDef {
    pub fn new(level: HirItemLevel, mutability: DefMutability) -> Self {
        Self { level, mutability }
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

pub type DefPubAccessibilityKind = HirPubAccessibilityKind;
pub type DefAccessibility = HirItemAccessibility;

#[derive(Debug, Clone)]
pub struct Definition {
    pub name: Symbol,
    pub kind: DefKind,

    // The definition id of the petal that this defintion belongs to.
    pub petal: Option<PetalId>,

    pub hir_id: HirId,
    pub span: Span,
    pub accessibility: DefAccessibility,
}

impl Definition {
    pub fn new(
        name: Symbol,
        kind: DefKind,
        petal: Option<PetalId>,
        hir_id: HirId,
        span: Span,
        accessibility: DefAccessibility,
    ) -> Self {
        Self {
            name,
            kind,
            petal,
            hir_id,
            span,
            accessibility,
        }
    }

    pub fn builtin(
        name: Symbol,
        kind: BuiltinKind,
        petal: Option<PetalId>,
        accessibility: DefAccessibility,
    ) -> Self {
        Self::new(
            name,
            DefKind::Builtin(kind),
            petal,
            HirId::Invalid,
            Span::default(),
            accessibility,
        )
    }

    pub fn new_default(
        name: Symbol,
        kind: DefKind,
        petal: Option<PetalId>,
        hir_id: HirId,
        span: Span,
    ) -> Self {
        Self::new(name, kind, petal, hir_id, span, DefAccessibility::Priv)
    }

    pub fn generic_params(&self) -> Option<&[DefId]> {
        match &self.kind {
            DefKind::Fn(fn_def) => Some(&fn_def.generic_params),

            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub def_id: DefId,
    pub accessibility: DefAccessibility,
}

impl Binding {
    pub fn new(def_id: DefId, accessibility: DefAccessibility) -> Self {
        Self {
            def_id,
            accessibility,
        }
    }
}
