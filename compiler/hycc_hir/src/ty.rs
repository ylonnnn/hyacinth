use hycc_span::Span;

use crate::{HirId, HirMutability, expr::HirExpr, path::HirPath};

#[derive(Debug, Clone)]
pub enum HirTyKind<'h> {
    Unit(Span),

    Path(&'h HirPath<'h>),
    Ref(Box<HirRef<'h>>),

    Array(Box<HirArray<'h>>),
    Slice(Box<HirSlice<'h>>),

    Tuple(Box<HirTuple<'h>>),
}

#[derive(Debug, Clone)]
pub struct HirTy<'h> {
    pub id: HirId,
    pub kind: HirTyKind<'h>,
    pub span: Span,
}

impl<'h> HirTy<'h> {
    pub fn new(kind: HirTyKind<'h>, span: Span) -> Self {
        Self {
            id: HirId::Invalid,
            kind,
            span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HirRef<'h> {
    pub ty: &'h HirTy<'h>,
    pub mutability: HirMutability,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirArray<'h> {
    pub size: &'h HirExpr<'h>,
    pub ty: &'h HirTy<'h>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirSlice<'h> {
    pub ty: &'h HirTy<'h>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirTuple<'h> {
    pub data: Vec<&'h HirTy<'h>>,
    pub span: Span,
}
