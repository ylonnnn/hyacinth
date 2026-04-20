use hycc_span::Span;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TyKind {
    Int(IntTy),
    Float(u8),

    Bool,

    Char,
    String,
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
