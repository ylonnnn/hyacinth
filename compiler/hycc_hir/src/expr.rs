use crate::path::HirPath;

use hycc_ast::expr::ExprEvaluatability;
use hycc_span::Span;

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum HirExprKind {
    Path(Box<HirPath>),
    Literal(Box<HirLiteral>),
    Binary(BinaryOp, Box<HirExpr>, Box<HirExpr>),
    Unary(Box<HirUnary>),
    Assign(Box<HirExpr>, Box<HirExpr>),
}

type HirExprEvaluatability = ExprEvaluatability;

#[derive(Debug, Clone)]
pub struct HirExpr {
    pub kind: HirExprKind,
    pub span: Span,
    pub eval: HirExprEvaluatability,
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
pub enum HirUnary {
    Pre(UnaryOp, Box<HirExpr>),
    Post(UnaryOp, Box<HirExpr>),
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
