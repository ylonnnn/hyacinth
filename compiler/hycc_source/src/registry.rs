use crate::source::{Source, SourceId};

#[derive(Debug)]
pub struct SourceRegistry {
    sources: Vec<Source>,
}

impl SourceRegistry {
    const CAPACITY: usize = u16::MAX as usize;

    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    pub fn root(&self) -> &Source {
        &self.sources[0]
    }

    pub fn register(&mut self, mut source: Source) -> SourceId {
        assert!(
            self.sources.len() < Self::CAPACITY,
            "project source tree cannot exceed {} source nodes!",
            Self::CAPACITY
        );

        let src_id = SourceId(self.sources.len() as u16);

        source.identifier.0 = src_id;
        self.sources.push(source);

        src_id
    }

    pub fn get(&self, id: SourceId) -> &Source {
        &self.sources[id.unwrap() as usize]
    }

    pub fn get_mut(&mut self, id: SourceId) -> &mut Source {
        &mut self.sources[id.unwrap() as usize]
    }
}
