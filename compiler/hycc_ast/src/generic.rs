use hycc_span::Span;

use crate::{Path, token::Token};

#[derive(Debug, Clone)]
pub struct GenericParamList {
    pub list: Vec<GenericParam>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenericParamKind {
    Ty,
    Const,
}

#[derive(Debug, Clone)]
pub struct GenericParam {
    pub ident: Token,
    pub proto_reqs: Vec<Path>,
    pub kind: GenericParamKind,
}

impl GenericParam {
    pub fn span(&self) -> Span {
        self.proto_reqs
            .last()
            .map_or(self.ident.span, |ident| self.ident.span.merge(ident.span))
    }
}
