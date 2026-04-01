use hycc_span::Span;
use hycc_symbol::Symbol;

use crate::{expr::HirExpr, ty::HirTy};

#[derive(Debug, Clone)]
pub struct HirRawIdent {
    pub ident: Symbol,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirIdent {
    pub ident: HirRawIdent,
    pub arguments: Option<HirIdentArguments>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum HirIdentArgument {
    Expr(Box<HirExpr>),
    Ty(Box<HirTy>),
}

#[derive(Debug, Clone)]
pub struct HirIdentArguments {
    pub data: Vec<HirIdentArgument>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirPath {
    pub segments: Vec<HirIdent>,
    pub span: Span,
}
