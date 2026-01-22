use crate::{
    core::Program,
    syntax::{GenNode, Item},
};

#[derive(Debug)]
pub struct ProgramNode<'program> {
    pub program: &'program mut Program,
    pub items: Vec<GenNode<Item>>,
}

impl<'program> ProgramNode<'program> {
    pub fn new(program: &'program mut Program, items: Vec<GenNode<Item>>) -> Self {
        Self { program, items }
    }
}
