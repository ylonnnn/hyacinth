use crate::{
    core::Span,
    syntax::{Expr, Stmt},
};

#[derive(Debug, Clone)]
pub struct Node<T> {
    pub node: T,
    pub span: Span,
}

pub type ExprNode = Node<Expr>;
pub type StmtNode = Node<Stmt>;
