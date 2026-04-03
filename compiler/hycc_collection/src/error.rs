use hycc_span::Span;
use hycc_symbol::Symbol;

#[derive(Debug, Clone)]
pub enum CollectionErrorKind {
    Duplication { ident: Symbol },
}

impl CollectionErrorKind {}

#[derive(Debug, Clone)]
pub struct CollectionError {
    pub kind: CollectionErrorKind,
    pub span: Span,
}

impl CollectionError {
    pub fn new(kind: CollectionErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}
