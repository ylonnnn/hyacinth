#[derive(Debug, Clone)]
pub enum LiteralExpr {
    Int(i64),
    Float(f64),
    Bool(bool),
    // TODO: Add other literal expression types
}
