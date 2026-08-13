use hycc_span::Span;

use crate::stmt::Stmt;

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}
