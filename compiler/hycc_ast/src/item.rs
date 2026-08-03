use std::path::PathBuf;

use crate::{
    Block, Expr, Identifier, Mutability, Path, Ty, generic::GenericParamList, token::Token,
};

use hycc_span::Span;
use hycc_util::ternary;

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum ItemKind {
    Refer(Box<Refer>),
    Petal(Box<Petal>),
    Proto(Box<Proto>),
    Extend(Box<Extend>),
    Struct(Box<Struct>),
    Fn(Box<Fn>),
    VarDecl(Box<VarDecl>),
}

impl ItemKind {
    pub fn span(&self) -> Span {
        match &self {
            Self::Refer(refer) => refer.span,
            Self::Petal(petal) => petal.span,
            Self::Proto(proto) => proto.span,
            Self::Extend(extend) => extend.span(),
            Self::Struct(strct) => strct.ident.span,
            Self::VarDecl(var) => var.span(),
            Self::Fn(func) => func.span(),
        }
    }

    pub fn kind(&self) -> &'static str {
        match &self {
            Self::Refer(_) => "reference/alias delcaration",
            Self::Petal(_) => "petal",
            Self::Proto(_) => "proto",
            Self::Extend(_) => "extend",
            Self::Struct(_) => "struct",
            Self::VarDecl(_) => "variable declaration",
            Self::Fn(_) => "function",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PubAccessibilityKind {
    All,
    Spathe,
    Super,
    This,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemAccessibility {
    Pub(PubAccessibilityKind),
    Priv,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemLevel {
    Top,
    Local(usize),
}

#[derive(Debug, Clone)]
pub struct Item {
    pub kind: ItemKind,
    pub span: Span,
    pub accessibility: ItemAccessibility,
    pub level: ItemLevel,
}

impl Item {
    pub fn new(kind: ItemKind, level: ItemLevel) -> Self {
        Self {
            span: kind.span(),
            kind,
            accessibility: ItemAccessibility::Priv,
            level,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Refer {
    pub target: ReferTarget,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ReferTargetKind {
    Child(Option<Token>),
    Parent(Vec<ReferTarget>),
}

#[derive(Debug, Clone)]
pub struct ReferTarget {
    pub symbol: Identifier,
    pub kind: ReferTargetKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum PetalKind {
    Root,
    File(Path, PathBuf),
    Inline(Path),
}

#[derive(Debug, Clone)]
pub struct Petal {
    pub kind: PetalKind,
    pub items: Vec<Item>,
    pub span: Span,
}

impl Petal {
    pub fn new(kind: PetalKind, items: Vec<Item>, span: Span) -> Self {
        Self { kind, items, span }
    }
}

#[derive(Debug, Clone)]
pub enum ProtoItemAssocFnKind {
    Sig(Box<FnSig>),
    Impl(Box<Item>),
}

#[derive(Debug, Clone)]
pub enum ProtoItem {
    // AssocTy(Box<Ty>),
    AssocConst(Box<Item>), // VarDecl Item
    AssocFn(ProtoItemAssocFnKind),
}

/// Protocol Node
#[derive(Debug, Clone)]
pub struct Proto {
    pub ident: Token,
    // TODO: generic_params
    pub items: Vec<ProtoItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Extend {
    pub target: Path,
    // TODO: with: Option<[PROTO]> // optional protocol implementation
    // TODO: if: Option<...> // conditional extension clause
    pub items: Vec<Item>,
}

impl Extend {
    pub fn span(&self) -> Span {
        self.target.span
    }
}

#[derive(Debug, Clone)]
pub struct Struct {
    pub ident: Token,
    pub generic_params: Option<GenericParamList>,
    pub fields: StructFieldList,
}

#[derive(Debug, Clone)]
pub struct StructFieldList {
    pub list: Vec<StructField>,
    pub span: Span,
}

pub type StructFieldAccessibility = ItemAccessibility;

#[derive(Debug, Clone)]
pub struct StructField {
    pub ident: Token,
    pub ty: Box<Ty>,
    pub accessibility: StructFieldAccessibility,
}

#[derive(Debug, Clone)]
pub struct FnSig {
    pub ident: Token,
    pub generic_params: Option<GenericParamList>,
    pub params: FnParamList,
    pub ret_ty: Option<Box<Ty>>,
}

impl FnSig {
    pub fn span(&self) -> Span {
        let end = ternary!(
            self.ret_ty.is_some(),
            self.ret_ty.as_ref().unwrap().span,
            self.params.span
        );

        self.ident.span.merge(end)
    }
}

#[derive(Debug, Clone)]
pub struct Fn {
    pub sig: FnSig,
    pub body: Block,
}

impl Fn {
    pub fn span(&self) -> Span {
        self.sig.span()
    }
}

#[derive(Debug, Clone)]
pub struct FnParamList {
    pub list: Vec<FnParam>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FnParam {
    pub ident: Token,
    pub ty: Box<Ty>,
}

impl FnParam {
    pub fn span(&self) -> Span {
        self.ident.span.merge(self.ty.span)
    }
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub ident: Token,
    pub mutability: Mutability,
    pub ty: Option<Box<Ty>>,
    pub val: Option<Box<Expr>>,
}

impl VarDecl {
    pub fn span(&self) -> Span {
        self.ident.span.merge(self.val.as_ref().map_or_else(
            || self.ty.as_ref().map_or_else(|| self.ident.span, |t| t.span),
            |v| v.span,
        ))
    }
}
