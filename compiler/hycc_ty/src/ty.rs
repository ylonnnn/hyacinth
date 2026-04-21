use hycc_hir::def::DefId;
use hycc_span::Span;

use crate::context::{TyId, TyVarId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TyKind {
    Unit,

    Int(IntTy),
    Float(u8),

    Bool,

    Char,
    String,

    Adt(DefId),

    Infer(TyVarId, InferKind),
    Param(DefId),
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
pub struct Ty {
    pub kind: TyKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TyVar {
    Unbound,
    Bound(TyId),
    Linked(TyVarId),
}
