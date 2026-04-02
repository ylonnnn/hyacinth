pub mod builder;
pub mod def;

pub mod expr;
pub mod item;
pub mod path;
pub mod program;
pub mod ty;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HirId(usize);

impl HirId {
    #[allow(non_snake_case)]
    pub fn Invalid() -> Self {
        Self(usize::MAX)
    }

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "hir id is not valid");
        self.0
    }
}
