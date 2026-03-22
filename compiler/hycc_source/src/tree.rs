use crate::source::{Source, SourceNode};

#[derive(Debug)]
pub struct SourceTree {
    pub root: SourceNode,

    counter: u16,
}

impl SourceTree {
    const CAPACITY: usize = (u16::MAX - 1) as usize;

    pub fn new(mut root: Source) -> Self {
        let counter = (root.identifier.0 = 0, root.identifier.0 + 1).1;
        Self {
            root: SourceNode::new(root),
            counter,
        }
    }

    pub fn append(_source: Source) {
        todo!()
    }
}
