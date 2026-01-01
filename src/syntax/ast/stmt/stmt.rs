use crate::syntax::{Expr, VariableDeclStmt};

pub enum Stmt {
    Expr(Expr),
    Let(VariableDeclStmt),
}
