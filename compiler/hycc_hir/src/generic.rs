use hycc_ast::generic::GenericParamKind;
use hycc_span::Span;

use crate::{
    HirId,
    path::{HirPath, HirRawIdent},
};

#[derive(Debug, Clone)]
pub struct HirGenericParamList<'h> {
    pub list: Vec<&'h HirGenericParam<'h>>,
    pub span: Span,
}

pub type HirGenericParamKind = GenericParamKind;

#[derive(Debug, Clone)]
pub struct HirGenericParam<'h> {
    pub id: HirId,
    pub ident: &'h HirRawIdent,
    pub proto_reqs: Vec<&'h HirPath<'h>>,
    pub kind: HirGenericParamKind,
    pub span: Span,
}

impl<'h> HirGenericParam<'h> {
    pub fn new(
        ident: &'h HirRawIdent,
        proto_reqs: Vec<&'h HirPath<'h>>,
        kind: HirGenericParamKind,
        span: Span,
    ) -> Self {
        Self {
            id: HirId::Invalid,
            ident,
            proto_reqs,
            kind,
            span,
        }
    }
}
