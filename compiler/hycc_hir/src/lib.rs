use hycc_arena::typed::TypedArena;

use crate::{
    block::HirBlock,
    expr::HirExpr,
    item::{HirFnParam, HirItem},
    path::{HirIdent, HirPath, HirRawIdent},
    stmt::HirStmt,
    ty::HirTy,
};

pub mod builder;
pub mod def;

pub mod block;
pub mod expr;
pub mod item;
pub mod path;
pub mod stmt;
pub mod ty;

#[derive(Debug)]
pub enum HirNode<'h> {
    Item(HirItem<'h>),

    Block(HirBlock<'h>),
    Stmt(HirStmt<'h>),

    Ty(HirTy<'h>),
    Expr(HirExpr<'h>),

    Path(HirPath<'h>),

    // Non-general nodes
    Ident(HirIdent<'h>),
    RawIdent(HirRawIdent),

    FnParam(HirFnParam<'h>),
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

        match node {
            HirNode::Item(node) => node.id = id,
            HirNode::Block(node) => node.id = id,
            HirNode::Stmt(node) => node.id = id,
            HirNode::Ty(node) => node.id = id,
            HirNode::Expr(node) => node.id = id,
            HirNode::Path(node) => node.id = id,

            HirNode::Ident(node) => node.id = id,
            HirNode::RawIdent(node) => node.id = id,
            HirNode::FnParam(node) => node.id = id,
        };

        id
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
