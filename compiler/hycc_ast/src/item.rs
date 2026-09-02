use std::path::PathBuf;

use crate::{
    Mutability,
    block::Block,
    expr::Expr,
    generic::GenericParamList,
    path::{Identifier, Path},
    token::Token,
    ty::Ty,
};

use hycc_span::Span;
use hycc_util::ternary;

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum ItemKind {
    Refer(Box<Refer>),
    Petal(Box<Petal>),
    Intf(Box<Intf>),
    Extend(Box<Extend>),
    Struct(Box<Struct>),
    FnDecl(Box<FnSig>),
    Fn(Box<Fn>),
    VarDecl(Box<VarSig>),
    VarDef(Box<VarDef>),
}

impl ItemKind {
    pub fn span(&self) -> Span {
        match &self {
            Self::Refer(refer) => refer.span,
            Self::Petal(petal) => petal.span,
            Self::Intf(intf) => intf.span,
            Self::Extend(extend) => extend.span(),
            Self::Struct(strct) => strct.ident.span,
            Self::FnDecl(func) => func.span(),
            Self::Fn(func) => func.span(),
            Self::VarDecl(var) => var.span(),
            Self::VarDef(var) => var.span(),
        }
    }

    pub fn kind(&self) -> &'static str {
        match &self {
            Self::Refer(_) => "reference/alias delcaration",
            Self::Petal(_) => "petal",
            Self::Intf(_) => "intf",
            Self::Extend(_) => "extend",
            Self::Struct(_) => "struct",
            Self::FnDecl(_) => "function declaration",
            Self::Fn(_) => "function",
            Self::VarDecl(_) => "variable declaration",
            Self::VarDef(_) => "variable definition",
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
pub enum IntfItem {
    Fn(Box<Item>),
    Var(Box<Item>),
}

/// interface Node
#[derive(Debug, Clone)]
pub struct Intf {
    pub ident: Token,
    pub generic_params: Option<GenericParamList>,
    pub items: Vec<IntfItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Extend {
    pub generic_params: Option<GenericParamList>,
    pub intf: Option<Path>,
    pub target: Ty,
    // TODO: if: Option<...> // conditional extension clause
    pub items: Vec<Item>,
}

impl Extend {
    pub fn span(&self) -> Span {
        self.intf.as_ref().map_or_else(
            || self.target.span,
            |intf| self.target.span.merge(intf.span),
        )
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
pub struct VarSig {
    pub ident: Token,
    pub ty: Option<Box<Ty>>,
    pub mutability: Mutability,
    pub is_comp: bool,
}

impl VarSig {
    pub fn span(&self) -> Span {
        self.ty
            .as_ref()
            .map_or_else(|| self.ident.span, |ty| self.ident.span.merge(ty.span))
    }
}

#[derive(Debug, Clone)]
pub struct VarDef {
    pub sig: VarSig,
    pub val: Option<Box<Expr>>,
}

impl VarDef {
    pub fn span(&self) -> Span {
        let sig_span = self.sig.span();
        self.val
            .as_ref()
            .map_or_else(|| sig_span, |val| sig_span.merge(val.span))
    }
}
