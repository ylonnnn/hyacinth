use crate::source::{Source, SourceId};

#[derive(Debug)]
pub struct SourceRegistry {
    sources: Vec<Source>,
}

impl SourceRegistry {
    const CAPACITY: usize = u16::MAX as usize;

    pub fn new(root: Source) -> Self {
        let mut inst = Self {
            sources: Vec::new(),
        };

        inst.register(root);
        inst
    }

    pub fn root(&self) -> &Source {
        &self.sources[0]
    }

    pub fn register(&mut self, mut source: Source) {
        assert!(
            self.sources.len() < Self::CAPACITY,
            "project source tree cannot exceed {} source nodes!",
            Self::CAPACITY
        );

        source.identifier.0 = SourceId(self.sources.len() as u16);
        self.sources.push(source);
    }

    pub fn get(&self, id: SourceId) -> &Source {
        &self.sources[id.unwrap() as usize]
    }

    pub fn get_mut(&mut self, id: SourceId) -> &mut Source {
        &mut self.sources[id.unwrap() as usize]
    }
}
