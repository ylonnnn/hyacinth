use hycc_source::{Source, SourceTree};

#[derive(Debug)]
pub struct Session {
    pub source_tree: SourceTree,
}

impl Session {
    pub fn new(root: Source) -> Self {
        Self {
            source_tree: SourceTree::new(root),
        }
    }
}
