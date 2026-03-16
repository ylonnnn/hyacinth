use crate::syntax::{Expr, Item};

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    Item(Item),
}
