use crate::{
    HirId, HirMutability,
    path::{HirPath, HirRawIdent},
};

use hycc_ast::expr::ExprEvaluatability;
use hycc_span::Span;

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum HirExprKind<'h> {
    Path(&'h HirPath<'h>),
    RefExpr(Box<HirRefExpr<'h>>),

    Literal(Box<HirLiteral>),

    Binary(BinaryOp, &'h HirExpr<'h>, &'h HirExpr<'h>),
    Unary(Box<HirUnary<'h>>),

    Assign(&'h HirExpr<'h>, &'h HirExpr<'h>),

    Array(Box<HirArrayExpr<'h>>),
    Struct(Box<HirStructExpr<'h>>),
}

type HirExprEvaluatability = ExprEvaluatability;

#[derive(Debug, Clone)]
pub struct HirExpr<'h> {
    pub id: HirId,
    pub kind: HirExprKind<'h>,
    pub span: Span,
    pub eval: HirExprEvaluatability,
}

impl<'h> HirExpr<'h> {
    pub fn new(kind: HirExprKind<'h>, span: Span, eval: HirExprEvaluatability) -> Self {
        Self {
            id: HirId::Invalid,
            kind,
            span,
            eval,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HirRefExpr<'h> {
    pub expr: &'h HirExpr<'h>,
    pub mutability: HirMutability,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum HirLiteral {
    Int { data: u64, is_negative: bool },
    Float(f64),
    Bool(bool),
    Char(u8),
    String(String),
}

#[derive(Debug, Clone)]
pub enum HirUnary<'h> {
    Pre(UnaryOp, &'h HirExpr<'h>),
    Post(UnaryOp, &'h HirExpr<'h>),
}

#[derive(Debug, Default, Clone, Copy)]
pub enum UnaryOp {
    #[default]
    Nop,

    Negative,
    Not,
    BitwiseNot,

    Increment,
    Decrement,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum BinaryOp {
    #[default]
    Nop,

    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,

    Eq,
    Neq,
    Less,
    LessEq,
    Greater,
    GreaterEq,

    And,
    Or,
    Xor, // TODO: maybe logical XOR?

    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseLShift,
    BitwiseRShift,
}

#[derive(Debug, Clone)]
pub struct HirArrayExpr<'h> {
    pub elements: Vec<&'h HirExpr<'h>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirStructExpr<'h> {
    pub path: &'h HirPath<'h>,
    pub fields: Vec<&'h HirStructExprField<'h>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirStructExprField<'h> {
    pub id: HirId,
    pub ident: &'h HirRawIdent,
    pub val: &'h HirExpr<'h>,
}

impl<'h> HirStructExprField<'h> {
    pub fn new(ident: &'h HirRawIdent, expr: &'h HirExpr<'h>) -> Self {
        Self {
            id: HirId::Invalid,
            ident,
            val: expr,
        }
    }

    pub fn span(&self) -> Span {
        self.ident.span.merge(&self.val.span)
    }
}
