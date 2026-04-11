use hycc_arena::typed::TypedArena;

use crate::{
    block::HirBlock,
    expr::HirExpr,
    item::HirItem,
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
    Ident(HirIdent<'h>),
    RawIdent(HirRawIdent),
}

#[derive(Debug)]
pub struct HirTable<'h>(TypedArena<HirNode<'h>>);

impl<'h> HirTable<'h> {
    pub fn new() -> Self {
        Self(TypedArena::new())
    }

    pub fn insert(&self, node: HirNode<'h>) -> HirId {
        self.0.alloc(node);
        HirId(unsafe { (&*self.0.data.get()).len() })
    }

    pub fn add(&self, node: HirNode<'h>) -> &'h HirNode<'h> {
        let data = unsafe { &mut *self.0.data.get() };
        data.push(Box::new(node));
        unsafe { &*(data.last().unwrap().as_ref() as *const HirNode<'h>) }
    }

    pub fn get(&'h self, id: HirId) -> &'h HirNode<'h> {
        unsafe { (&*self.0.data.get())[id.unwrap()].as_ref() }
    }

    pub fn get_mut(&'h self, id: HirId) -> &'h mut HirNode<'h> {
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
