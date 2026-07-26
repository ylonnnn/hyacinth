use std::{collections::HashMap, fmt::Display};

use hycc_hir::def::DefId;
use hycc_span::Span;
use hycc_ty::context::TyId;

// #[derive(Debug, Clone)]
// pub struct DeclTable {
//     data: Vec<Decl>,
//     defs: HashMap<DefId, DeclId>,
// }

// impl DeclTable {
//     pub fn new() -> Self {
//         Self {
//             data: Vec::new(),
//             defs: HashMap::new(),
//         }
//     }
// }

#[derive(Debug, Clone)]
pub enum DeclKind {
    Local(LocalDeclKind),
    Global,
}

#[derive(Debug, Clone)]
pub enum LocalDeclKind {
    Ret,
    Param,
    Var,
    Temp,
}

#[derive(Debug, Clone)]
pub struct Decl {
    pub ty: TyId,
    pub span: Span,
    pub kind: DeclKind,
    pub mutability: Mutability,
}

impl Decl {
    pub fn new(kind: DeclKind, ty: TyId, mutability: Mutability, span: Span) -> Self {
        Self {
            ty,
            span,
            kind,
            mutability,
        }
    }

    pub fn local(local_kind: LocalDeclKind, ty: TyId, mutability: Mutability, span: Span) -> Self {
        Self::new(DeclKind::Local(local_kind), ty, mutability, span)
    }

    pub fn global(ty: TyId, mutability: Mutability, span: Span) -> Self {
        Self::new(DeclKind::Global, ty, mutability, span)
    }
}

pub type Mutability = hycc_hir::HirMutability;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalDeclId(pub(crate) usize);

impl LocalDeclId {
    #[allow(non_upper_case_globals)]
    pub const Invalid: Self = Self(usize::MAX);

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "local decl id is not valid!");
        self.0
    }
}

impl Display for LocalDeclId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "_{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalDeclId(pub(crate) usize);

impl GlobalDeclId {
    #[allow(non_upper_case_globals)]
    pub const Invalid: Self = Self(usize::MAX);

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "global decl id is not valid!");
        self.0
    }
}

impl Display for GlobalDeclId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "_g{}", self.0)
    }
}
