use std::{fmt::Display, sync::Arc};

use hycc_hir::def::DefId;
use hycc_span::Span;

use crate::context::{TyId, TyVarId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RefMutability {
    Mutable,
    Immutable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessKind {
    Ref(RefMutability),
    Owned,
}

impl AccessKind {
    pub fn allows(self, req_access: Self) -> bool {
        match (self, req_access) {
            (_, Self::Owned) => self == Self::Owned,
            (Self::Ref(RefMutability::Immutable), Self::Ref(RefMutability::Mutable)) => false,
            _ => true,
        }
    }
}

impl Display for AccessKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Self::Owned => String::from("owned"),
                Self::Ref(mutability) => format!(
                    "{} reference",
                    match &mutability {
                        RefMutability::Immutable => "shared",
                        RefMutability::Mutable => "mutable",
                    }
                ),
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TyKind {
    Unit,
    Never,

    Int(IntTy),
    Float(u8),

    Bool,

    Char,
    String,

    Array(TyId /* TODO: constant size*/),
    Slice(TyId),

    Tuple(Box<Arc<[TyId]>>),

    Ref(TyId, RefMutability),

    Fn(Box<FnTy>),

    Adt(DefId),

    Infer(TyVarId, InferKind),
    Param(DefId),
}

#[derive(Debug, Clone)]
pub struct Ty {
    pub id: TyId,
    pub span: Span,
}

impl Ty {
    pub fn new(id: TyId, span: Span) -> Self {
        Self { id, span }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnTy {
    pub params: Arc<[TyId]>,
    pub ret_ty: TyId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InferKind {
    Any,

    Int,
    Float,
}

impl InferKind {
    pub fn compatible(&self, ty: &TyKind) -> bool {
        match self {
            InferKind::Any => true,
            InferKind::Int => matches!(ty, TyKind::Int(_) | TyKind::Infer(_, InferKind::Int)),
            InferKind::Float => matches!(ty, TyKind::Float(_) | TyKind::Infer(_, InferKind::Float)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IntTy {
    Fixed(u8, bool),
    Size(bool),
}

#[derive(Debug, Clone)]
pub enum TyVar {
    Unbound,
    Bound(TyId),
    Linked(TyVarId),
}
