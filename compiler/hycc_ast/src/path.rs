use crate::{
    core::Span,
    syntax::{Expr, Token, Ty},
};

#[derive(Debug, Clone)]
pub struct Identifier {
    pub ident: Token,
    pub arguments: Vec<IdentifierArgument>,
}

impl Identifier {
    pub fn new(ident: Token, arguments: Vec<IdentifierArgument>) -> Self {
        Self { ident, arguments }
    }
}

#[derive(Debug, Clone)]
pub enum IdentifierArgumentKind {
    Expr(Expr),
    Ty(Ty),
}

#[derive(Debug, Clone)]
pub struct IdentifierArgument {
    pub kind: IdentifierArgumentKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub segments: Vec<Identifier>,
    pub span: Span,
}

impl Path {
    pub fn new(segments: Vec<Identifier>) -> Self {
        Self {
            segments,
            span: Span::default(), // TODO: adjust path spaan
        }
    }

    pub fn add(&mut self, segment: Identifier) {
        self.segments.push(segment);
    }
}
