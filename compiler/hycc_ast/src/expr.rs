use crate::{Path, token::Token};

use hycc_span::Span;

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum ExprKind {
    Path(Path),
    Literal(LiteralExpr),
    Binary(Token, Box<Expr>, Box<Expr>),
    Unary(Unary),
}

impl ExprKind {
    pub fn span(&self) -> Span {
        match self {
            Self::Path(path) => path.span,
            Self::Literal(expr) => expr.span(),
            Self::Binary(_, left, right) => left.span.merge(&right.span),
            Self::Unary(expr) => expr.span(),
        }
    }

    pub fn eval(&self) -> ExprEvaluatability {
        match self {
            Self::Path(..) => ExprEvaluatability::Unknown,
            Self::Literal(..) => ExprEvaluatability::CompileTime,
            Self::Binary(_, left, right) => left.eval.infer(&right.eval),
            Self::Unary(expr) => expr.eval(),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExprEvaluatability {
    RunTime,
    Unknown,
    CompileTime,
}

impl ExprEvaluatability {
    pub fn infer(&self, other: &Self) -> Self {
        if *self == Self::RunTime || *other == Self::RunTime {
            Self::RunTime
        } else if *self == Self::Unknown || *other == Self::Unknown {
            Self::Unknown
        } else {
            Self::CompileTime
        }
    }
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
    pub eval: ExprEvaluatability,
}

impl Expr {
    pub fn new(kind: ExprKind) -> Self {
        Self {
            span: kind.span(),
            eval: kind.eval(),
            kind,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum LiteralExpr {
    Int(Token),
    Float(Token),
    Bool(Token),
    // TODO: Add other literal expression types
}

impl LiteralExpr {
    pub fn span(&self) -> Span {
        match self {
            Self::Int(tok) => tok.span,
            Self::Float(tok) => tok.span,
            Self::Bool(tok) => tok.span,
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Unary {
    Pre(Token, Box<Expr>),
    Post(Token, Box<Expr>),
}

impl Unary {
    pub fn span(&self) -> Span {
        match self {
            Self::Pre(tok, expr) => tok.span.merge(&expr.span),
            Self::Post(tok, expr) => expr.span.merge(&tok.span),
        }
    }

    pub fn eval(&self) -> ExprEvaluatability {
        match self {
            Self::Pre(_, expr) | Self::Post(_, expr) => expr.eval,
        }
    }
}
