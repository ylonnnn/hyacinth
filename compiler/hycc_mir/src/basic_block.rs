use crate::{stmt::MirStatement, term::MirTerminator};

#[derive(Debug, Clone)]
pub struct MirBasicBlock {
    pub statements: Vec<MirStatement>,
    pub terminator: Option<MirTerminator>,
}

impl MirBasicBlock {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
            terminator: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MirBasicBlockId(pub(crate) usize);

impl MirBasicBlockId {
    #[allow(non_upper_case_globals)]
    pub const Invalid: Self = Self(usize::MAX);

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "mir basic block id is not valid!");
        self.0
    }
}
