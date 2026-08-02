use crate::{
    HirId, HirMutability,
    block::HirBlock,
    path::{HirIdent, HirPath, HirRawIdent},
    ty::HirTy,
};

use hycc_ast::expr::ExprEvaluatability;
use hycc_const::table::ConstId;
use hycc_span::Span;
use hycc_symbol::Symbol;

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum HirExprKind<'h> {
    Path(&'h HirPath<'h>),
    RefExpr(Box<HirRefExpr<'h>>),

    Literal(Box<HirLiteral>),

    Binary(BinaryOp, &'h HirExpr<'h>, &'h HirExpr<'h>),
    Unary(Box<HirUnary<'h>>),

    Assign(&'h HirExpr<'h>, &'h HirExpr<'h>),

    Block(&'h HirBlock<'h>),

    Array(Box<HirArrayExpr<'h>>),
    Tuple(Box<HirTupleExpr<'h>>),
    Struct(Box<HirStructExpr<'h>>),

    AnonFn(Box<HirAnonFn<'h>>),

    FnCall(Box<HirFnCall<'h>>),

    FieldAccess(Box<HirFieldAccess<'h>>),
    MethodCall(Box<HirMethodCall<'h>>),

    If(Box<HirIfExpr<'h>>),
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
pub struct HirLiteral(pub(crate) ConstId);

impl HirLiteral {
    pub fn const_id(&self) -> ConstId {
        self.0
    }
}

// #[derive(Debug, Clone)]
// pub enum HirLiteral {
//     Int(u64),
//     Float(f64),
//     Bool(bool),
//     Char(u8),
//     String(String),
// }

#[derive(Debug, Clone)]
pub enum HirUnary<'h> {
    Pre(UnaryOp, &'h HirExpr<'h>),
    Post(UnaryOp, &'h HirExpr<'h>),
}

#[derive(Debug, Default, Clone, Copy)]
pub enum UnaryOp {
    #[default]
    Nop,

    Negate,
    Not,

    Deref,
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
pub struct HirTupleExpr<'h> {
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
        self.ident.span.merge(self.val.span)
    }
}

#[derive(Debug, Clone)]
pub struct HirAnonFnParam<'h> {
    pub id: HirId,
    pub ident: &'h HirRawIdent,
    pub ty: Option<&'h HirTy<'h>>,
    pub span: Span,
}

impl<'h> HirAnonFnParam<'h> {
    pub fn new(ident: &'h HirRawIdent, ty: Option<&'h HirTy<'h>>) -> Self {
        Self {
            id: HirId::Invalid,
            ident,
            ty,
            span: ident.span.merge(ty.map(|ty| ty.span).unwrap_or(ident.span)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HirAnonFnParamList<'h> {
    pub list: Vec<&'h HirAnonFnParam<'h>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirAnonFn<'h> {
    pub params: HirAnonFnParamList<'h>,
    pub ret_ty: Option<&'h HirTy<'h>>,
    pub body: &'h HirBlock<'h>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirCallArguments<'h> {
    pub data: Vec<&'h HirExpr<'h>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirFnCall<'h> {
    pub callee: &'h HirExpr<'h>,
    pub arguments: HirCallArguments<'h>,
}

#[derive(Debug, Clone, Copy)]
pub enum HirFieldAccessFieldKind {
    Ident(Symbol),
    Index(usize),
}

#[derive(Debug, Clone)]
pub struct HirFieldAccessField {
    pub kind: HirFieldAccessFieldKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirFieldAccess<'h> {
    pub leading: &'h HirExpr<'h>,
    pub field: HirFieldAccessField,
}

#[derive(Debug, Clone)]
pub struct HirMethodCall<'h> {
    pub receiver: &'h HirExpr<'h>,
    pub callee: &'h HirIdent<'h>,
    pub arguments: HirCallArguments<'h>,
}

#[derive(Debug, Clone)]
pub struct HirIfExpr<'h> {
    pub span: Span,
    pub cond: &'h HirExpr<'h>,
    pub consequent: &'h HirBlock<'h>,
    pub alternate: Option<&'h HirBlock<'h>>,
}
