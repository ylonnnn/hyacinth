use hycc_span::Span;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TyKind {
    Int(IntTy),

    Bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IntTy {
    Fixed(u8, bool),
    Size(bool),
}

#[derive(Debug, Clone)]
pub struct Ty {
    pub kind: TyKind,
    pub span: Span,
}
