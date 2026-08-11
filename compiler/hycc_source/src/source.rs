use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceId(pub(crate) u16);

impl SourceId {
    #[allow(non_upper_case_globals)]
    pub const Invalid: Self = Self(u16::MAX);

    pub fn data(&self) -> u16 {
        self.0
    }

    pub fn is_valid(&self) -> bool {
        self.0 != u16::MAX
    }

    pub fn unwrap(&self) -> u16 {
        assert!(self.is_valid(), "source id is not valid!");
        self.0
    }
}

impl Default for SourceId {
    fn default() -> Self {
        Self::Invalid
    }
}

impl From<u16> for SourceId {
    fn from(value: u16) -> Self {
        Self(value)
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
            Ok(data) => match std::path::absolute(path) {
                Ok(path) => Self {
                    identifier: (SourceId::Invalid, path.to_str().unwrap().into()),
                    data,
                },
                Err(err) => {
                    panic!("error: {err:?}")
                }
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
