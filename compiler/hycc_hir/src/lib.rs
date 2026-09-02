use hycc_arena::typed::TypedArena;
use hycc_ast::Mutability;

use crate::{
    block::HirBlock,
    expr::{HirAnonFnParam, HirExpr, HirStructExprField},
    generic::HirGenericParam,
    item::{HirFnParam, HirFnSig, HirItem, HirStructField, HirVarSig},
    path::{HirIdent, HirPath, HirRawIdent},
    stmt::HirStmt,
    ty::HirTy,
};

pub mod builder;
pub mod def;
pub mod petal;
pub mod scope;

pub mod block;
pub mod expr;
pub mod generic;
pub mod item;
pub mod path;
pub mod stmt;
pub mod ty;

pub type HirMutability = Mutability;

#[derive(Debug)]
pub enum HirNode<'h> {
    Item(HirItem<'h>),

    Block(HirBlock<'h>),
    Stmt(HirStmt<'h>),

    Ty(HirTy<'h>),
    Expr(HirExpr<'h>),

    Path(HirPath<'h>),

    // // Sub-Item Nodes
    // FnSig(HirFnSig<'h>),
    // VarSig(HirVarSig<'h>),

    // Non-general nodes
    Ident(HirIdent<'h>),
    RawIdent(HirRawIdent),

    StructField(HirStructField<'h>),
    GenericParam(HirGenericParam<'h>),
    FnParam(HirFnParam<'h>),
    AnonFnParam(HirAnonFnParam<'h>),

    StructExprField(HirStructExprField<'h>),
}

impl<'h> HirNode<'h> {
    pub fn get_item(&self) -> Option<&HirItem<'h>> {
        match &self {
            Self::Item(item) => Some(&item),
            _ => None,
        }
    }

    pub fn expect_item(&self) -> &HirItem<'h> {
        self.get_item().expect("expected an Item")
    }
}

#[derive(Debug)]
pub struct HirTable<'h>(TypedArena<HirNode<'h>>);

impl<'h> HirTable<'h> {
    pub fn new() -> Self {
        Self(TypedArena::new())
    }

    fn attach_id(&self, node: &mut HirNode<'h>) -> HirId {
        let data = unsafe { &*self.0.data.get() };
        let id = HirId(data.len());

        let n_id = match node {
            HirNode::Item(node) => &mut node.id,
            HirNode::Block(node) => &mut node.id,
            HirNode::Stmt(node) => &mut node.id,
            HirNode::Ty(node) => &mut node.id,
            HirNode::Expr(node) => &mut node.id,
            HirNode::Path(node) => &mut node.id,

            HirNode::Ident(node) => &mut node.id,
            HirNode::RawIdent(node) => &mut node.id,
            HirNode::StructField(node) => &mut node.id,
            HirNode::GenericParam(node) => &mut node.id,
            HirNode::FnParam(node) => &mut node.id,
            HirNode::AnonFnParam(node) => &mut node.id,
            HirNode::StructExprField(node) => &mut node.id,
        };

        (*n_id = id, id).1
    }

    pub fn insert(&self, mut node: HirNode<'h>) -> HirId {
        let hir_id = self.attach_id(&mut node);
        self.0.alloc(node);

        hir_id
    }

    pub fn add(&self, node: HirNode<'h>) -> &'h HirNode<'h> {
        let hir_id = self.insert(node);
        self.get(hir_id)
    }

    pub fn get(&self, id: HirId) -> &'h HirNode<'h> {
        unsafe { (&*self.0.data.get())[id.unwrap()].as_ref() }
    }

    pub fn get_mut(&self, id: HirId) -> &'h mut HirNode<'h> {
        unsafe { (&mut *self.0.data.get())[id.unwrap()].as_mut() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HirId(usize);

impl HirId {
    #[allow(non_upper_case_globals)]
    pub const Invalid: Self = Self(usize::MAX);

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "hir id is not valid");
        self.0
    }
}
