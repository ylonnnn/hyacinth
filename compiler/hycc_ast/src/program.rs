use crate::{core::Program, syntax::Item};

#[derive(Debug)]
pub struct ProgramNode<'program> {
    pub program: &'program mut Program,
    pub items: Vec<Item>,
}

impl<'program> ProgramNode<'program> {
    pub fn new(program: &'program mut Program, items: Vec<Item>) -> Self {
        Self { program, items }
    }
}
