use hycc_ast::item::{ItemAccessibility, StructFieldAccessibility};
use hycc_span::Span;

use crate::{
    HirId, HirMutability, block::HirBlock, expr::HirExpr, path::{HirPath, HirRawIdent}, ty::HirTy
};

#[derive(Debug, Clone)]
pub enum HirItemKind<'h> {
    Petal(Box<HirPetal<'h>>),
    Struct(Box<HirStruct<'h>>),
    Fn(Box<HirFn<'h>>),
    VarDecl(Box<HirVarDecl<'h>>),
}

pub type HirItemAccessibility = ItemAccessibility;

#[derive(Debug, Clone)]
pub struct HirItem<'h> {
    pub id: HirId,
    pub kind: HirItemKind<'h>,
    pub span: Span,
    pub accessibility: HirItemAccessibility,
}

impl<'h> HirItem<'h> {
    pub fn new(kind: HirItemKind<'h>, span: Span) -> Self {
        Self {
            id: HirId::Invalid,
            kind,
            span,
            accessibility: HirItemAccessibility::Priv,
        }
    }
}

#[derive(Debug, Clone)]
pub enum HirPetalKind<'h> {
    Root,
    File(&'h HirPath<'h>),
    Inline(&'h HirPath<'h>),
}

#[derive(Debug, Clone)]
pub struct HirPetal<'h> {
    pub kind: HirPetalKind<'h>,
    pub items: Vec<&'h HirItem<'h>>,
    pub span: Span,
}

impl<'h> HirPetal<'h> {
    pub fn is_inline(&self) -> bool {
        matches!(self.kind, HirPetalKind::Inline(..))
    }
}

#[derive(Debug, Clone)]
pub struct HirStruct<'h> {
    pub ident: &'h HirRawIdent,
    pub fields: HirStructFieldList<'h>,
}

#[derive(Debug, Clone)]
pub struct HirStructFieldList<'h> {
    pub list: Vec<&'h HirStructField<'h>>,
    pub span: Span,
}

pub type HirStructFieldAccessibility = StructFieldAccessibility;

#[derive(Debug, Clone)]
pub struct HirStructField<'h> {
    pub id: HirId,
    pub ident: &'h HirRawIdent,
    pub ty: &'h HirTy<'h>,
    pub accessibility: HirStructFieldAccessibility,
    pub span: Span,
}

impl<'h> HirStructField<'h> {
    pub fn new(
        ident: &'h HirRawIdent,
        ty: &'h HirTy<'h>,
        accessibility: HirStructFieldAccessibility,
        span: Span,
    ) -> Self {
        Self {
            id: HirId::Invalid,
            ident,
            ty,
            accessibility,
            span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HirFn<'h> {
    pub ident: &'h HirRawIdent,
    pub params: HirFnParamList<'h>,
    pub ret_ty: Option<&'h HirTy<'h>>,
    pub body: &'h HirBlock<'h>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirFnParamList<'h> {
    pub list: Vec<&'h HirFnParam<'h>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirFnParam<'h> {
    pub id: HirId,
    pub ident: &'h HirRawIdent,
    pub ty: &'h HirTy<'h>,
    pub span: Span,
}

impl<'h> HirFnParam<'h> {
    pub fn new(ident: &'h HirRawIdent, ty: &'h HirTy<'h>, span: Span) -> Self {
        Self {
            id: HirId::Invalid,
            ident,
            ty,
            span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HirVarDecl<'h> {
    pub ident: &'h HirRawIdent,
    pub mutability: HirMutability,
    pub ty: Option<&'h HirTy<'h>>,
    pub val: Option<&'h HirExpr<'h>>,
    pub span: Span,
}
