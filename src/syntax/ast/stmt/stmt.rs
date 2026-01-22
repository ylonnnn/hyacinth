use crate::syntax::Expr;

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
}
