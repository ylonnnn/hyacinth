use std::fmt::Display;

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

impl Display for MirBasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for statement in &self.statements {
            writeln!(f, "  {}", &statement)?;
        }

        if let Some(term) = &self.terminator {
            writeln!(f, "  {}", &term)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MirBasicBlockId(pub(crate) usize);

impl MirBasicBlockId {
    #[allow(non_upper_case_globals)]
    pub const Invalid: Self = Self(usize::MAX);

    pub fn is_valid(&self) -> bool {
        self.0 != usize::MAX
    }

    pub fn unwrap(&self) -> usize {
        assert!(self.is_valid(), "mir basic block id is not valid!");
        self.0
    }
}

impl Display for MirBasicBlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bb{}", self.0)
    }
}
