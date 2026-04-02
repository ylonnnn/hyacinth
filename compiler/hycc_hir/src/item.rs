use hycc_span::Span;

use crate::{HirId, path::HirRawIdent, ty::HirTy};

#[derive(Debug, Clone)]
pub enum HirItemKind {
    Fn(Box<HirFn>),
    Var,
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
    // TODO: pub ty: HirTy,
    pub span: Span,
}
