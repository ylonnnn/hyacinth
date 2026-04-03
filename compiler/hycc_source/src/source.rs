use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceId(pub u16);

impl SourceId {
    #[allow(non_snake_case)]
    pub fn Invalid() -> Self {
        Self(u16::MAX)
    }

    pub fn is_valid(&self) -> bool {
        self.0 != u16::MAX
    }

    pub fn unwrap(&self) -> u16 {
        assert!(self.is_valid(), "source id is not valid!");
        self.0
    }
}

#[derive(Debug)]
pub struct Source {
    pub identifier: (SourceId, String),
    pub data: String,
}

impl Source {
    pub fn new(path: &str) -> Self {
        match fs::read_to_string(path.to_string()) {
            Ok(data) => Self {
                identifier: (SourceId::Invalid(), path.into()),
                data,
            },
            Err(err) => match err {
                _ => panic!("error: {err:?}"),
            },
        }
    }

    pub fn is_registered(&self) -> bool {
        self.identifier.0.is_valid()
    }
}
