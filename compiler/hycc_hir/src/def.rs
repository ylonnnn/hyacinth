use std::{collections::HashMap, fmt::Display};

use hycc_span::Span;
use hycc_symbol::Symbol;

use crate::{
    HirId, HirMutability,
    generic::HirGenericParamKind,
    item::{HirItemAccessibility, HirItemLevel, HirPubAccessibilityKind},
    petal::PetalId,
};

#[derive(Debug, Clone, Copy)]
pub enum DefResolution {
    Petal(DefId),
    Ty(DefId),
    Value(DefId),
}

impl DefResolution {
    pub fn def_id(&self) -> DefId {
        match &self {
            Self::Petal(def_id) | Self::Ty(def_id) | Self::Value(def_id) => *def_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DefNodeResolution {
    pub base: DefResolution,
    pub unresolved: usize,
}

#[derive(Debug)]
pub struct DefinitionTable {
    map: HashMap<HirId, DefId>,
    pub res_map: HashMap<HirId, DefNodeResolution>,
    data: Vec<Definition>,
    pub builtins: Vec<DefId>,
}

impl DefinitionTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            res_map: HashMap::new(),
            data: Vec::new(),
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

    pub fn attach_res(&mut self, hir_id: HirId, res: DefNodeResolution) {
        self.res_map.insert(hir_id, res);
    }

    pub fn get_res(&self, hir_id: HirId) -> Option<&DefNodeResolution> {
        self.res_map.get(&hir_id)
    }

    pub fn expect_res(&self, hir_id: HirId) -> &DefNodeResolution {
        self.get_res(hir_id).expect(&format!(
            "expected a resolution attached to hir id {hir_id:?}"
        ))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DefResKind {
    Petal,
    Ty,
    Value,
}

#[derive(Debug, Clone)]
pub enum DefKind {
    Builtin(BuiltinKind),

    Petal,
    intf,

    Alias(Box<DefKind>),

    Adt(Box<AdtDef>),

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
            | Self::intf
            | Self::Alias(_)
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
            Self::intf => String::from("interface"),

            Self::Alias(_) => String::from("alias"),

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
                BuiltinKind::SelfTy | BuiltinKind::Ty(_) => DefSpace::Type,
            };
        }

        match self {
            Self::Builtin(_) => unreachable!(),
            Self::Alias(data) => data.space(),

            Self::Petal | Self::intf | Self::Adt(_) | Self::GenericParam(_) => DefSpace::Type,

            Self::Fn(_) | Self::FnParam | Self::Var(_) => DefSpace::Value,
        }
    }

    pub fn res_kind(&self) -> DefResKind {
        match &self {
            DefKind::Petal => DefResKind::Petal,

            DefKind::intf
            | DefKind::Builtin(BuiltinKind::SelfTy)
            | DefKind::Builtin(BuiltinKind::Ty(_))
            | DefKind::Adt(_) => DefResKind::Ty,

            DefKind::Alias(def_kind) => def_kind.res_kind(),

            DefKind::GenericParam(gp_def) => match &gp_def.kind {
                HirGenericParamKind::Ty => DefResKind::Ty,
                _ => DefResKind::Value,
            },

            _ => DefResKind::Value,
        }
    }

    pub fn get_builtin(&self) -> Option<&BuiltinKind> {
        match &self {
            Self::Builtin(kind) => Some(kind),
            _ => None,
        }
    }

    pub fn expect_builtin(&self) -> &BuiltinKind {
        self.get_builtin()
            .expect("expect definition kind to be Builtin")
    }

    pub fn get_adt(&self) -> Option<&AdtDef> {
        match &self {
            Self::Adt(adt_def) => Some(adt_def),
            _ => None,
        }
    }

    pub fn get_mut_adt(&mut self) -> Option<&mut AdtDef> {
        match self {
            Self::Adt(adt_def) => Some(adt_def),
            _ => None,
        }
    }

    pub fn expect_adt(&self) -> &AdtDef {
        self.get_adt()
            .expect("expected definition kind to be an Adt")
    }

    pub fn expect_mut_adt(&mut self) -> &mut AdtDef {
        self.get_mut_adt()
            .expect("expected definition kind to be an Adt")
    }

    pub fn get_generic_param(&self) -> Option<&GenericParamDef> {
        match &self {
            Self::GenericParam(def) => Some(&def),
            _ => None,
        }
    }

    pub fn expect_generic_param(&self) -> &GenericParamDef {
        self.get_generic_param()
            .expect("expected definition kind to be a GenericParam")
    }

    pub fn get_fn(&self) -> Option<&FnDef> {
        match &self {
            Self::Fn(def) => Some(&def),
            _ => None,
        }
    }

    pub fn get_mut_fn(&mut self) -> Option<&mut FnDef> {
        match self {
            Self::Fn(def) => Some(def),
            _ => None,
        }
    }

    pub fn expect_fn(&self) -> &FnDef {
        self.get_fn().expect("expected definition kind to be an Fn")
    }

    pub fn expect_mut_fn(&mut self) -> &mut FnDef {
        self.get_mut_fn()
            .expect("expected definition kind to be an Fn")
    }

    pub fn get_var(&self) -> Option<&VarDef> {
        match &self {
            Self::Var(def) => Some(&def),
            _ => None,
        }
    }

    pub fn expect_var(&self) -> &VarDef {
        self.get_var()
            .expect("expected definition kin dto be a Var")
    }
}

#[derive(Debug, Clone)]
pub enum BuiltinKind {
    SelfTy,
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
pub struct intfDef {}

#[derive(Debug, Clone)]
pub enum intfDefItem {}

#[derive(Debug, Clone)]
pub enum AdtKind {
    Struct(StructDef),
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
pub struct AdtDef {
    pub kind: AdtKind,
    pub generic_params: Vec<DefId>,
}

impl AdtDef {
    pub fn new(kind: AdtKind) -> Self {
        Self {
            kind,
            generic_params: Vec::new(),
        }
    }

    pub fn article(&self) -> &'static str {
        self.kind.article()
    }

    pub fn kind(&self) -> &'static str {
        self.kind.kind()
    }

    pub fn get_struct(&self) -> Option<&StructDef> {
        #[allow(irrefutable_let_patterns)]
        if let AdtKind::Struct(struct_def) = &self.kind {
            Some(&struct_def)
        } else {
            None
        }
    }

    pub fn get_mut_struct(&mut self) -> Option<&mut StructDef> {
        #[allow(irrefutable_let_patterns)]
        if let AdtKind::Struct(struct_def) = &mut self.kind {
            Some(struct_def)
        } else {
            None
        }
    }

    pub fn expect_struct(&self) -> &StructDef {
        self.get_struct()
            .expect(&format!("expected internal definition to be a struct!"))
    }

    pub fn expect_mut_struct(&mut self) -> &mut StructDef {
        self.get_mut_struct()
            .expect(&format!("expected internal definition to be a struct!"))
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
    pub data: usize, // 4 bytes for depth; 4 bytes idx
    pub kind: HirGenericParamKind,
}

impl GenericParamDef {
    pub fn new(depth: u32, idx: u32, kind: HirGenericParamKind) -> Self {
        Self {
            data: (depth as usize) << u32::BITS | (idx as usize),
            kind,
        }
    }

    pub fn depth(&self) -> u32 {
        (self.data >> u32::BITS) as u32
    }

    pub fn idx(&self) -> u32 {
        (self.data & (u32::MAX as usize)) as u32
    }
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
    // The definition id of the petal that this defintion belongs to.
    pub petal: Option<PetalId>,
    pub kind: DefKind,
    pub name: Symbol,
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
            DefKind::Adt(adt_kind) => Some(&adt_kind.generic_params),

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
