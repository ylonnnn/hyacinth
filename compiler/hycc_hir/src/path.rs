use hycc_span::Span;
use hycc_symbol::Symbol;

use crate::{HirId, expr::HirExpr, ty::HirTy};

#[derive(Debug, Clone)]
pub struct HirRawIdent {
    pub id: HirId,
    pub ident: Symbol,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirIdent {
    pub id: HirId,
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
    pub id: HirId,
    pub segments: Vec<HirIdent>,
    pub span: Span,
}
