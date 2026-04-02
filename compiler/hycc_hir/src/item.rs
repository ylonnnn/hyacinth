use hycc_span::Span;

use crate::{HirId, block::HirBlock, expr::HirExpr, path::HirRawIdent, ty::HirTy};

#[derive(Debug, Clone)]
pub enum HirItemKind {
    Fn(Box<HirFn>),
    VarDecl(Box<HirVarDecl>),
}

#[derive(Debug, Clone)]
pub struct HirItem {
    pub id: HirId,
    pub kind: HirItemKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirFn {
    pub ident: HirRawIdent,
    pub params: HirFnParamList,
    pub ret_ty: Option<HirTy>,
    pub body: HirBlock,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirFnParamList {
    pub list: Vec<HirFnParam>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirFnParam {
    pub ident: HirRawIdent,
    pub ty: Box<HirTy>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirVarDecl {
    pub id: HirId,
    pub ident: HirRawIdent,
    pub ty: Option<Box<HirTy>>,
    pub val: Option<Box<HirExpr>>,
    pub span: Span,
}
