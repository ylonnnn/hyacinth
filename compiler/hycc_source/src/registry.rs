use crate::source::Source;

#[derive(Debug)]
pub struct SourceRegistry {
    pub sources: Vec<Source>,
}

impl SourceRegistry {
    const CAPACITY: usize = (u16::MAX - 1) as usize;

    pub fn new(entry: Source) -> Self {
        let mut inst = Self {
            sources: Vec::with_capacity(4),
        };

        inst.register(entry);
        inst
    }

    pub fn entry(&mut self) -> &mut Source {
        &mut self.sources[0]
    }

    pub fn register(&mut self, mut source: Source) -> &mut Source {
        assert!(
            self.sources.len() <= Self::CAPACITY,
            "source registry cannot hold more than {} sources!",
            Self::CAPACITY
        );
        let id = self.sources.len() as u16;

        source.identifier.0 = id;
        self.sources.push(source);

        &mut self.sources[id as usize]
    }
}
