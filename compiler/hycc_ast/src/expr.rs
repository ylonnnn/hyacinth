use crate::{Block, Identifier, Mutability, Path, Ty, token::Token};

use hycc_span::Span;

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum ExprKind {
    Path(Box<Path>),
    RefExpr(Box<RefExpr>),

    Literal(Token),

    Binary(Token, Box<Expr>, Box<Expr>),
    Unary(Box<Unary>),

    Assign(Box<Expr>, Box<Expr>),

    Block(Box<Block>),

    Array(Box<ArrayExpr>),
    Tuple(Box<TupleExpr>),
    Struct(Box<StructExpr>),

    AnonFn(Box<AnonFn>),

    FnCall(Box<FnCall>),

    FieldAccess(Box<FieldAccess>),
    MethodCall(Box<MethodCall>),

    If(Box<IfExpr>),
}

impl ExprKind {
    pub fn span(&self) -> Span {
        match self {
            Self::Path(path) => path.span,
            Self::RefExpr(reference) => reference.span,

            Self::Literal(expr) => expr.span,

            Self::Binary(_, left, right) => left.span.merge(right.span),
            Self::Unary(expr) => expr.span(),

            Self::Assign(left, right) => left.span.merge(right.span),

            Self::Block(block) => block.span,

            Self::Array(array) => array.span,
            Self::Tuple(tup) => tup.span,
            Self::Struct(strct) => strct.span,

            Self::AnonFn(anfn) => anfn.span,

            Self::FnCall(call) => call.callee.span.merge(call.arguments.span),

            Self::FieldAccess(access) => access.leading.span.merge(access.field.span),
            Self::MethodCall(call) => call.receiver.span.merge(call.arguments.span),

            Self::If(ite) => ite.span,
        }
    }

    pub fn eval(&self) -> ExprEvaluatability {
        match self {
            Self::Path(..) => ExprEvaluatability::Unknown,
            Self::RefExpr(reference) => reference.expr.eval,

            Self::Literal(..) => ExprEvaluatability::CompileTime,

            Self::Binary(_, left, right) => left.eval.infer(&right.eval),
            Self::Unary(expr) => expr.eval(),

            Self::Assign(..) => ExprEvaluatability::RunTime,

            Self::Block(_) => ExprEvaluatability::Unknown,

            Self::Array(array) => array
                .elements
                .iter()
                .map(|el| el.eval)
                .reduce(|acc, eval| acc.infer(&eval))
                .unwrap_or_else(|| ExprEvaluatability::CompileTime),

            Self::Tuple(tup) => tup
                .elements
                .iter()
                .map(|el| el.eval)
                .reduce(|acc, eval| acc.infer(&eval))
                .unwrap_or_else(|| ExprEvaluatability::CompileTime),

            Self::Struct(strct) => strct
                .fields
                .iter()
                .map(|f| f.val.eval)
                .reduce(|acc, eval| acc.infer(&eval))
                .unwrap_or_else(|| ExprEvaluatability::CompileTime),

            Self::AnonFn(_) => ExprEvaluatability::Unknown,

            Self::FnCall(_) => ExprEvaluatability::Unknown,

            Self::FieldAccess(_) => ExprEvaluatability::Unknown,
            Self::MethodCall(_) => ExprEvaluatability::Unknown,

            // TODO: improve?
            Self::If(_) => ExprEvaluatability::Unknown,
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

#[derive(Debug, Clone)]
pub struct RefExpr {
    pub expr: Box<Expr>,
    pub mutability: Mutability,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Unary {
    Pre(Token, Box<Expr>),
    Post(Token, Box<Expr>),
}

impl Unary {
    pub fn span(&self) -> Span {
        match self {
            Self::Pre(tok, expr) | Self::Post(tok, expr) => tok.span.merge(expr.span),
        }
    }

    pub fn eval(&self) -> ExprEvaluatability {
        match self {
            Self::Pre(_, expr) | Self::Post(_, expr) => expr.eval,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArrayExpr {
    pub elements: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TupleExpr {
    pub elements: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructExpr {
    pub path: Path,
    pub fields: Vec<StructExprField>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructExprField {
    pub ident: Token,
    pub val: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct FieldAccess {
    pub leading: Box<Expr>,
    pub field: Token,
}

#[derive(Debug, Clone)]
pub struct AnonFnParam {
    pub ident: Token,
    pub ty: Option<Box<Ty>>,
}

#[derive(Debug, Clone)]
pub struct AnonFnParamList {
    pub list: Vec<AnonFnParam>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AnonFn {
    pub params: AnonFnParamList,
    pub ret_ty: Option<Box<Ty>>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CallArguments {
    pub data: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FnCall {
    pub callee: Box<Expr>,
    pub arguments: CallArguments,
}

#[derive(Debug, Clone)]
pub struct MethodCall {
    pub receiver: Box<Expr>,
    pub callee: Identifier,
    pub arguments: CallArguments,
}

#[derive(Debug, Clone)]
pub struct IfExpr {
    pub span: Span,
    pub cond: Box<Expr>,
    pub consequent: Box<Block>,
    pub alternate: Option<Box<Block>>,
}
