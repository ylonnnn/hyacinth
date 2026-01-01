use crate::core::program::program::Program;

#[derive(Debug)]
pub struct ProgramRegistry {
    pub programs: Vec<Program>,
    entry: usize,
}

impl ProgramRegistry {
    pub fn new(entry: Program) -> Self {
        Self {
            programs: vec![entry],
            entry: 0,
        }
    }
}
