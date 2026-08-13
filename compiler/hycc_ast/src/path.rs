use crate::{expr::Expr, token::Token, ty::Ty};

use hycc_span::Span;
use hycc_util::ternary;

#[derive(Debug, Clone)]
pub struct Identifier {
    pub ident: Token,
    pub arguments: Option<IdentifierArguments>,
    pub span: Span,
}

impl Identifier {
    pub fn new(ident: Token, arguments: Option<IdentifierArguments>) -> Self {
        Self {
            span: ternary!(
                arguments.is_some(),
                ident.span.merge(arguments.as_ref().unwrap().span),
                ident.span
            ),
            ident,
            arguments,
        }
    }
}

#[derive(Debug, Clone)]
pub enum IdentifierArgument {
    Expr(Box<Expr>),
    Ty(Box<Ty>),
}

#[derive(Debug, Clone)]
pub struct IdentifierArguments {
    pub data: Vec<IdentifierArgument>,
    pub span: Span,
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
            front.span.merge(back.span)
        } else {
            Span::default()
        };

        Self { segments, span }
    }

    pub fn add(&mut self, segment: Identifier) {
        self.span = self.span.merge(segment.span);
        self.segments.push(segment);
    }
}
