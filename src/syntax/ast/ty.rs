use crate::{core::Span, syntax::Path};

#[derive(Debug, Clone)]
pub enum TyKind {
    Path(Path),
}

#[derive(Debug, Clone)]
pub struct Ty {
    pub kind: TyKind,
    pub span: Span,
}
