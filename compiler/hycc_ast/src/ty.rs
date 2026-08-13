use crate::{
    Mutability,
    expr::Expr,
    path::{Identifier, Path},
    token::Token,
};

use hycc_span::Span;

#[derive(Debug, Clone)]
pub enum TyKind {
    Unit(Span),

    Path(Box<Path>),
    Ref(Box<Ref>),

    Array(Box<Array>),
    Slice(Box<Slice>),

    Tuple(Box<Tuple>),

    Fn(Box<FnTy>),
}

impl TyKind {
    pub fn span(&self) -> Span {
        match self {
            Self::Unit(span) => *span,
            Self::Path(path) => path.span,
            Self::Ref(reference) => reference.span,
            Self::Array(arr) => arr.span,
            Self::Slice(slice) => slice.span,
            Self::Tuple(tup) => tup.span,
            Self::Fn(func) => func.span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ty {
    pub kind: TyKind,
    pub span: Span,
}

impl Ty {
    pub fn new(kind: TyKind) -> Self {
        Self {
            span: kind.span(),
            kind,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ref {
    pub ty: Box<Ty>,
    pub mutability: Mutability,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Array {
    pub size: Box<Expr>,
    pub ty: Box<Ty>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Slice {
    pub ty: Box<Ty>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Tuple {
    pub data: Vec<Ty>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FnTy {
    pub params: Vec<Ty>,
    pub ret_ty: Option<Ty>,
    pub span: Span,
}
