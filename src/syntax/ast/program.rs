use crate::{
    core::Program,
    syntax::{Item, SpannedNode},
};

#[derive(Debug)]
pub struct ProgramNode<'program> {
    pub program: &'program mut Program,
    pub items: Vec<SpannedNode<Item>>,
}

impl<'program> ProgramNode<'program> {
    pub fn new(program: &'program mut Program, items: Vec<SpannedNode<Item>>) -> Self {
        Self { program, items }
    }
}
