use crate::{
    core::Span,
    syntax::{Expr, Item, Path, Stmt, Type},
};

#[derive(Debug, Clone)]
pub enum NodeKind {
    None,
    Expr(Expr),
    Type(Type),
    Stmt(Stmt),
    Item(Item),
    Path(Path),
}

impl From<Expr> for NodeKind {
    fn from(value: Expr) -> Self {
        Self::Expr(value)
    }
}

#[derive(Debug, Clone)]
pub struct SpannedNode<T> {
    pub node: T,
    pub span: Span,
}

impl<T> SpannedNode<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}

pub type Node = SpannedNode<NodeKind>;

impl Default for NodeKind {
    fn default() -> Self {
        Self::None
    }
}

impl Node {
    pub fn from_expr(expr: Expr, span: Span) -> Self {
        Self {
            node: NodeKind::Expr(expr),
            span,
        }
    }

    pub fn from_ty(ty: Type, span: Span) -> Self {
        Self {
            node: NodeKind::Type(ty),
            span,
        }
    }

    pub fn from_stmt(stmt: Stmt, span: Span) -> Self {
        Self {
            node: NodeKind::Stmt(stmt),
            span,
        }
    }

    pub fn from_item(item: Item, span: Span) -> Self {
        Self {
            node: NodeKind::Item(item),
            span,
        }
    }

    pub fn from_path(path: Path, span: Span) -> Self {
        Self {
            node: NodeKind::Path(path),
            span,
        }
    }

    pub fn expr(&mut self) -> Option<Expr> {
        if let NodeKind::Expr(expr) = std::mem::take(&mut self.node) {
            Some(expr)
        } else {
            None
        }
    }

    pub fn expr_node(&mut self) -> Option<SpannedNode<Expr>> {
        Some(SpannedNode::new(self.expr()?, self.span.clone()))
    }

    pub fn stmt(&mut self) -> Option<Stmt> {
        if let NodeKind::Stmt(stmt) = std::mem::take(&mut self.node) {
            Some(stmt)
        } else {
            None
        }
    }

    pub fn stmt_node(&mut self) -> Option<SpannedNode<Stmt>> {
        Some(SpannedNode::new(self.stmt()?, self.span.clone()))
    }

    pub fn item(&mut self) -> Option<Item> {
        if let NodeKind::Item(item) = std::mem::take(&mut self.node) {
            Some(item)
        } else {
            None
        }
    }

    pub fn item_node(&mut self) -> Option<SpannedNode<Item>> {
        Some(SpannedNode::new(self.item()?, self.span.clone()))
    }

    pub fn path(&mut self) -> Option<Path> {
        if let NodeKind::Path(path) = std::mem::take(&mut self.node) {
            Some(path)
        } else {
            None
        }
    }

    pub fn path_node(&mut self) -> Option<SpannedNode<Path>> {
        Some(SpannedNode::new(self.path()?, self.span.clone()))
    }

    pub fn ty(&mut self) -> Option<Type> {
        if let NodeKind::Type(ty) = std::mem::take(&mut self.node) {
            Some(ty)
        } else {
            None
        }
    }

    pub fn ty_node(&mut self) -> Option<SpannedNode<Type>> {
        Some(SpannedNode::new(self.ty()?, self.span.clone()))
    }
}
