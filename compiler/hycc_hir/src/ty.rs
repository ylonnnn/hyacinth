use hycc_span::Span;

use crate::{expr::HirExpr, path::HirPath};

#[derive(Debug, Clone)]
pub enum HirTyKind {
    Path(Box<HirPath>),
    Array(Box<HirArray>),
    Unit(Span),
}

#[derive(Debug, Clone)]
pub struct HirTy {
    pub kind: HirTyKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirArray {
    pub size: Box<HirExpr>,
    pub ty: Box<HirTy>,
    pub span: Span,
}
