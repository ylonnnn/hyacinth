use crate::{Expr, Ty, token::Token};

use hycc_span::Span;

#[derive(Debug, Clone)]
pub enum ItemKind {
    VarDecl(VarDeclStmt),
}

impl ItemKind {
    pub fn span(&self) -> Span {
        match self {
            Self::VarDecl(var) => var.span(),
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
pub struct VarDeclStmt {
    pub ident: Token,
    pub comp: VarDeclComposition,
}

#[derive(Debug, Clone)]
pub enum VarDeclComposition {
    TypeAnnotated(Ty),
    ValueInitialized(Expr),
    Full(Ty, Expr),
}

impl VarDeclStmt {
    pub fn span(&self) -> Span {
        self.ident.span.merge(&match &self.comp {
            VarDeclComposition::TypeAnnotated(ty) => ty.span,
            VarDeclComposition::ValueInitialized(expr) | VarDeclComposition::Full(_, expr) => {
                expr.span
            }
        })
    }
}
