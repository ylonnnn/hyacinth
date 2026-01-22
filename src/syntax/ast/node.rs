use crate::{
    core::Span,
    syntax::{Expr, Item, Stmt},
};

#[derive(Debug, Clone)]
pub enum NodeData {
    None,
    Expr(Expr),
    Stmt(Stmt),
    Item(Item),
}

impl From<Expr> for NodeData {
    fn from(value: Expr) -> Self {
        Self::Expr(value)
    }
}

#[derive(Debug, Clone)]
pub struct GenNode<T> {
    pub node: T,
    pub span: Span,
}

impl<T> GenNode<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}

pub type Node = GenNode<NodeData>;

impl Default for NodeData {
    fn default() -> Self {
        Self::None
    }
}

impl Node {
    pub fn from_expr(expr: Expr, span: Span) -> Self {
        Self {
            node: NodeData::Expr(expr),
            span,
        }
    }

    pub fn from_stmt(stmt: Stmt, span: Span) -> Self {
        Self {
            node: NodeData::Stmt(stmt),
            span,
        }
    }

    pub fn from_item(item: Item, span: Span) -> Self {
        Self {
            node: NodeData::Item(item),
            span,
        }
    }

    pub fn expr(&mut self) -> Option<Expr> {
        if let NodeData::Expr(expr) = std::mem::take(&mut self.node) {
            Some(expr)
        } else {
            None
        }
    }

    pub fn expr_node(&mut self) -> Option<GenNode<Expr>> {
        Some(GenNode::new(self.expr()?, self.span.clone()))
    }

    pub fn stmt(&mut self) -> Option<Stmt> {
        if let NodeData::Stmt(stmt) = std::mem::take(&mut self.node) {
            Some(stmt)
        } else {
            None
        }
    }

    pub fn stmt_node(&mut self) -> Option<GenNode<Stmt>> {
        Some(GenNode::new(self.stmt()?, self.span.clone()))
    }

    pub fn item(&mut self) -> Option<Item> {
        if let NodeData::Item(item) = std::mem::take(&mut self.node) {
            Some(item)
        } else {
            None
        }
    }

    pub fn item_node(&mut self) -> Option<GenNode<Item>> {
        Some(GenNode::new(self.item()?, self.span.clone()))
    }
}
