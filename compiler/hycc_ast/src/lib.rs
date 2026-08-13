pub mod block;
pub mod expr;
pub mod generic;
pub mod item;
pub mod path;
pub mod stmt;
pub mod token;
pub mod token_stream;
pub mod ty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Mutable,
    Immutable,
}
