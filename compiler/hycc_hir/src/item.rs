use hycc_span::Span;

use crate::{HirId, block::HirBlock, expr::HirExpr, path::HirRawIdent, ty::HirTy};

#[derive(Debug, Clone)]
pub enum HirItemKind<'h> {
    Fn(Box<HirFn<'h>>),
    VarDecl(Box<HirVarDecl<'h>>),
}

#[derive(Debug, Clone)]
pub struct HirItem<'h> {
    pub id: HirId,
    pub kind: HirItemKind<'h>,
    pub span: Span,
}

impl<'h> HirItem<'h> {
    pub fn new(kind: HirItemKind<'h>, span: Span) -> Self {
        Self {
            id: HirId::Invalid,
            kind,
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
    pub list: Vec<HirFnParam<'h>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirFnParam<'h> {
    pub ident: &'h HirRawIdent,
    pub ty: &'h HirTy<'h>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirVarDecl<'h> {
    pub ident: &'h HirRawIdent,
    pub ty: Option<&'h HirTy<'h>>,
    pub val: Option<&'h HirExpr<'h>>,
    pub span: Span,
}
