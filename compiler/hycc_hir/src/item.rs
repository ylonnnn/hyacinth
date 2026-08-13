use hycc_ast::item::{
    ItemAccessibility, ItemLevel, PubAccessibilityKind, StructFieldAccessibility,
};
use hycc_span::Span;
use hycc_symbol::Symbol;

use crate::{
    HirId, HirMutability,
    block::HirBlock,
    expr::HirExpr,
    generic::HirGenericParamList,
    path::{HirIdent, HirPath, HirRawIdent},
    ty::HirTy,
};

#[derive(Debug, Clone)]
pub enum HirItemKind<'h> {
    Refer(Box<HirRefer<'h>>),
    Petal(Box<HirPetal<'h>>),
    Proto(Box<HirProto<'h>>),
    Extend(Box<HirExtend<'h>>),
    Struct(Box<HirStruct<'h>>),
    Fn(Box<HirFn<'h>>),
    VarDecl(Box<HirVarDecl<'h>>),
}

pub type HirPubAccessibilityKind = PubAccessibilityKind;
pub type HirItemAccessibility = ItemAccessibility;
pub type HirItemLevel = ItemLevel;

#[derive(Debug, Clone)]
pub struct HirItem<'h> {
    pub id: HirId,
    pub kind: HirItemKind<'h>,
    pub span: Span,
    pub accessibility: HirItemAccessibility,
    pub level: HirItemLevel,
}

impl<'h> HirItem<'h> {
    pub fn new(kind: HirItemKind<'h>, level: HirItemLevel, span: Span) -> Self {
        Self {
            id: HirId::Invalid,
            kind,
            span,
            accessibility: HirItemAccessibility::Priv,
            level,
        }
    }

    pub fn is_top_level(&self) -> bool {
        self.level == HirItemLevel::Top
    }

    pub fn get_refer(&self) -> Option<&HirRefer> {
        match &self.kind {
            HirItemKind::Refer(refer) => Some(&refer),
            _ => None,
        }
    }

    pub fn expect_refer(&self) -> &HirRefer {
        self.get_refer().expect("expected to be Refer")
    }

    pub fn get_petal(&self) -> Option<&HirPetal> {
        match &self.kind {
            HirItemKind::Petal(petal) => Some(&petal),
            _ => None,
        }
    }

    pub fn expect_petal(&self) -> &HirPetal {
        self.get_petal().expect("expected to be Petal")
    }

    pub fn get_proto(&self) -> Option<&HirProto> {
        match &self.kind {
            HirItemKind::Proto(proto) => Some(&proto),
            _ => None,
        }
    }

    pub fn expect_proto(&self) -> &HirProto {
        self.get_proto().expect("expected to be Proto")
    }

    pub fn get_extend(&self) -> Option<&HirExtend> {
        match &self.kind {
            HirItemKind::Extend(extend) => Some(&extend),
            _ => None,
        }
    }

    pub fn expect_extend(&self) -> &HirExtend {
        self.get_extend().expect("expected to be Extend")
    }

    pub fn get_struct(&self) -> Option<&HirStruct> {
        match &self.kind {
            HirItemKind::Struct(strct) => Some(&strct),
            _ => None,
        }
    }

    pub fn expect_struct(&self) -> &HirStruct {
        self.get_struct().expect("expected to be Struct")
    }

    pub fn get_fn(&self) -> Option<&HirFn> {
        match &self.kind {
            HirItemKind::Fn(func) => Some(&func),
            _ => None,
        }
    }

    pub fn expect_fn(&self) -> &HirFn {
        self.get_fn().expect("expected to be Fn")
    }

    pub fn get_var(&self) -> Option<&HirVarDecl> {
        match &self.kind {
            HirItemKind::VarDecl(decl) => Some(&decl),
            _ => None,
        }
    }

    pub fn expect_var(&self) -> &HirVarDecl {
        self.get_var().expect("expected to be Var")
    }
}

#[derive(Debug, Clone)]
pub struct HirRefer<'h> {
    pub target: HirReferTarget<'h>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum HirReferTargetKind<'h> {
    Child(Option<Symbol>),
    Parent(Vec<HirReferTarget<'h>>),
}

#[derive(Debug, Clone)]
pub struct HirReferTarget<'h> {
    pub symbol: &'h HirIdent<'h>,
    pub kind: HirReferTargetKind<'h>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum HirPetalKind<'h> {
    Root,
    File(&'h HirPath<'h>),
    Inline(&'h HirPath<'h>),
}

#[derive(Debug, Clone)]
pub struct HirPetal<'h> {
    pub kind: HirPetalKind<'h>,
    pub items: Vec<&'h HirItem<'h>>,
    pub span: Span,
}

impl<'h> HirPetal<'h> {
    pub fn is_inline(&self) -> bool {
        matches!(self.kind, HirPetalKind::Inline(..))
    }

    pub fn path(&self) -> Option<&'h HirPath<'h>> {
        match &self.kind {
            HirPetalKind::File(path) | HirPetalKind::Inline(path) => Some(&path),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum HirProtoItemAssocFnKind<'h> {
    Sig(HirFnSig<'h>),
    Impl(&'h HirItem<'h>),
}

#[derive(Debug, Clone)]
pub enum HirProtoItem<'h> {
    // AssocTy(&'h HirTy<'h>),
    AssocConst(&'h HirItem<'h>),
    AssocFn(HirProtoItemAssocFnKind<'h>),
}

#[derive(Debug, Clone)]
pub struct HirProto<'h> {
    pub ident: &'h HirRawIdent,
    // TODO: generic_params
    pub items: Vec<HirProtoItem<'h>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirExtend<'h> {
    pub target: &'h HirTy<'h>,
    pub generic_params: Option<HirGenericParamList<'h>>,
    // TODO: with: Option<[PROTO]>
    pub items: Vec<&'h HirItem<'h>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirStruct<'h> {
    pub ident: &'h HirRawIdent,
    pub generic_params: Option<HirGenericParamList<'h>>,
    pub fields: HirStructFieldList<'h>,
}

#[derive(Debug, Clone)]
pub struct HirStructFieldList<'h> {
    pub list: Vec<&'h HirStructField<'h>>,
    pub span: Span,
}

pub type HirStructFieldAccessibility = StructFieldAccessibility;

#[derive(Debug, Clone)]
pub struct HirStructField<'h> {
    pub id: HirId,
    pub ident: &'h HirRawIdent,
    pub ty: &'h HirTy<'h>,
    pub accessibility: HirStructFieldAccessibility,
    pub span: Span,
}

impl<'h> HirStructField<'h> {
    pub fn new(
        ident: &'h HirRawIdent,
        ty: &'h HirTy<'h>,
        accessibility: HirStructFieldAccessibility,
        span: Span,
    ) -> Self {
        Self {
            id: HirId::Invalid,
            ident,
            ty,
            accessibility,
            span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HirFnSig<'h> {
    pub ident: &'h HirRawIdent,
    pub generic_params: Option<HirGenericParamList<'h>>,
    pub params: HirFnParamList<'h>,
    pub ret_ty: Option<&'h HirTy<'h>>,
}

#[derive(Debug, Clone)]
pub struct HirFn<'h> {
    pub sig: HirFnSig<'h>,
    pub body: &'h HirBlock<'h>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirFnParamList<'h> {
    pub list: Vec<&'h HirFnParam<'h>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirFnParam<'h> {
    pub id: HirId,
    pub ident: &'h HirRawIdent,
    pub ty: &'h HirTy<'h>,
    pub span: Span,
}

impl<'h> HirFnParam<'h> {
    pub fn new(ident: &'h HirRawIdent, ty: &'h HirTy<'h>, span: Span) -> Self {
        Self {
            id: HirId::Invalid,
            ident,
            ty,
            span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HirVarDecl<'h> {
    pub ident: &'h HirRawIdent,
    pub mutability: HirMutability,
    pub ty: Option<&'h HirTy<'h>>,
    pub val: Option<&'h HirExpr<'h>>,
    pub span: Span,
}
