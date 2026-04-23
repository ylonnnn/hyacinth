use crate::{Expr, Path};

use hycc_span::Span;

#[derive(Debug, Clone)]
pub enum TyKind {
    Path(Box<Path>),

    Array(Box<Array>),
    Slice(Box<Slice>),

    Unit(Span),
}

impl TyKind {
    pub fn span(&self) -> Span {
        match self {
            Self::Path(path) => path.span,
            Self::Array(arr) => arr.span,
            Self::Slice(slice) => slice.span,
            Self::Unit(span) => *span,
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
