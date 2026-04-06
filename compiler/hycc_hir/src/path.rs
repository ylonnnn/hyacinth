use hycc_span::Span;
use hycc_symbol::Symbol;

use crate::{HirId, expr::HirExpr, ty::HirTy};

#[derive(Debug, Clone)]
pub struct HirRawIdent {
    pub id: HirId,
    pub ident: Symbol,
    pub span: Span,
}

impl HirRawIdent {
    pub fn new(ident: Symbol, span: Span) -> Self {
        Self {
            id: HirId::Invalid,
            ident,
            span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HirIdent<'h> {
    pub id: HirId,
    pub ident: &'h HirRawIdent,
    pub arguments: Option<HirIdentArguments<'h>>,
    pub span: Span,
}

impl<'h> HirIdent<'h> {
    pub fn new(
        ident: &'h HirRawIdent,
        arguments: Option<HirIdentArguments<'h>>,
        span: Span,
    ) -> Self {
        Self {
            id: HirId::Invalid,
            ident,
            arguments,
            span,
        }
    }
}

#[derive(Debug, Clone)]
pub enum HirIdentArgument<'h> {
    Expr(&'h HirExpr<'h>),
    Ty(&'h HirTy<'h>),
}

#[derive(Debug, Clone)]
pub struct HirIdentArguments<'h> {
    pub data: Vec<HirIdentArgument<'h>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirPath<'h> {
    pub id: HirId,
    pub segments: Vec<&'h HirIdent<'h>>,
    pub span: Span,
}

impl<'h> HirPath<'h> {
    pub fn new(segments: Vec<&'h HirIdent<'h>>, span: Span) -> Self {
        Self {
            id: HirId::Invalid,
            segments,
            span,
        }
    }
}
