use crate::syntax::{Expr, SpannedNode, Token, Type};

#[derive(Debug, Clone)]
pub struct Identifier {
    pub ident: Token,
    pub arguments: Vec<SpannedNode<IdentifierArgument>>,
}

impl Identifier {
    pub fn new(ident: Token, arguments: Vec<SpannedNode<IdentifierArgument>>) -> Self {
        Self { ident, arguments }
    }
}

#[derive(Debug, Clone)]
pub enum IdentifierArgument {
    Expr(Expr),
    Type(Type),
}

#[derive(Debug, Clone)]
pub struct Path {
    pub segments: Vec<SpannedNode<Identifier>>,
}

impl Path {
    pub fn new(segments: Vec<SpannedNode<Identifier>>) -> Self {
        Self { segments }
    }

    pub fn add(&mut self, segment: SpannedNode<Identifier>) {
        self.segments.push(segment);
    }
}
