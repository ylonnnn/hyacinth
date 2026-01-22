use crate::syntax::VariableDeclStmt;

#[derive(Debug, Clone)]
pub enum Item {
    Variable(VariableDeclStmt),
}
