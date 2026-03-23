use crate::{Expr, Ty, token::Token};

use hycc_span::Span;

#[derive(Debug, Clone)]
pub struct Identifier {
    pub ident: Token,
    pub arguments: IdentifierArguments,
    pub span: Span,
}

impl Identifier {
    pub fn new(ident: Token, arguments: IdentifierArguments) -> Self {
        Self {
            span: ident.span.merge(&arguments.span),
            ident,
            arguments,
        }
    }
}

#[derive(Debug, Clone)]
pub enum IdentifierArgument {
    Expr(Expr),
    Ty(Ty),
}

#[derive(Debug, Clone)]
pub struct IdentifierArguments {
    pub data: Vec<IdentifierArgument>,
    pub span: Span,
}

impl IdentifierArguments {
    pub fn new(data: Vec<IdentifierArgument>, span: Span) -> Self {
        Self { data, span }
    }
}

#[derive(Debug, Clone)]
pub struct Path {
    pub segments: Vec<Identifier>,
    pub span: Span,
}

impl Path {
    pub fn new(segments: Vec<Identifier>) -> Self {
        let span = if let Some(front) = segments.first()
            && let Some(back) = segments.last()
        {
            front.span.merge(&back.span)
        } else {
            Span::default()
        };

        Self { segments, span }
    }

    pub fn add(&mut self, segment: Identifier) {
        self.segments.push(segment);
    }
}
