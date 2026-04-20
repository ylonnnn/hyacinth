use hycc_span::Span;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TyKind {
    Bool,
}

#[derive(Debug, Clone)]
pub struct Ty {
    pub kind: TyKind,
    pub span: Span,
}
