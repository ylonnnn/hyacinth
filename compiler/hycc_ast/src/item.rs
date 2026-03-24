use crate::{Block, Expr, Ty, token::Token};

use hycc_span::Span;
use hycc_util::ternary;

#[derive(Debug, Clone)]
pub enum ItemKind {
    VarDecl(VarDecl),
    Fn(Fn),
}

impl ItemKind {
    pub fn span(&self) -> Span {
        match self {
            Self::VarDecl(var) => var.span(),
            Self::Fn(func) => func.span(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub kind: ItemKind,
    pub span: Span,
}

impl Item {
    pub fn new(kind: ItemKind) -> Self {
        Self {
            span: kind.span(),
            kind,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub ident: Token,
    pub comp: VarDeclComposition,
}

#[derive(Debug, Clone)]
pub enum VarDeclComposition {
    TypeAnnotated(Box<Ty>),
    ValueInitialized(Box<Expr>),
    Full(Box<Ty>, Box<Expr>),
}

impl VarDecl {
    pub fn span(&self) -> Span {
        self.ident.span.merge(&match &self.comp {
            VarDeclComposition::TypeAnnotated(ty) => ty.span,
            VarDeclComposition::ValueInitialized(expr) | VarDeclComposition::Full(_, expr) => {
                expr.span
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct Fn {
    pub ident: Token,
    pub params: FnParamList,
    pub ret_ty: Option<Box<Ty>>,
    pub body: Block,
}

impl Fn {
    pub fn span(&self) -> Span {
        let end = ternary!(
            self.ret_ty.is_some(),
            self.ret_ty.as_ref().unwrap().span,
            self.params.span
        );

        self.ident.span.merge(&end)
    }
}

#[derive(Debug, Clone)]
pub struct FnParam {
    pub ident: Token,
    pub ty: Box<Ty>,
}

impl FnParam {
    pub fn span(&self) -> Span {
        self.ident.span.merge(&self.ty.span)
    }
}

#[derive(Debug, Clone)]
pub struct FnParamList {
    pub list: Vec<Box<FnParam>>,
    pub span: Span,
}
