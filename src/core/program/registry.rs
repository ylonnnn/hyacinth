use crate::core::program::program::Program;

#[derive(Debug)]
pub struct ProgramRegistry {
    pub programs: Vec<Program>,
}

impl ProgramRegistry {
    pub fn new(entry: Program) -> Self {
        Self {
            programs: vec![entry],
        }
    }

    pub fn entry(&mut self) -> &mut Program {
        &mut self.programs[0]
    }
}
