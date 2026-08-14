use crate::source::{Source, SourceId};

#[derive(Debug)]
pub struct SourceRegistry {
    sources: Vec<Source>,
}

impl SourceRegistry {
    const CAPACITY: usize = u16::MAX as usize;

    pub fn new(mut source: Source) -> Self {
        source.identifier.0 = SourceId(0);
        Self {
            sources: vec![source],
        }
    }

    pub fn sources(&self) -> Vec<SourceId> {
        self.sources.iter().map(|src| src.identifier.0).collect()
    }

    pub fn root(&self) -> (SourceId, &Source) {
        let id = SourceId(0);
        (id, &self.sources[id.unwrap() as usize])
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
