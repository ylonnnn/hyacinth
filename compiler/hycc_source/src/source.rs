use std::fs;

#[derive(Debug)]
pub struct Source {
    pub identifier: (u16, String),
    pub data: String,
}

impl Source {
    pub fn new(path: &str) -> Self {
        match fs::read_to_string(path.to_string()) {
            Ok(data) => Self {
                identifier: (u16::MAX, path.into()),
                data,
            },
            Err(err) => match err {
                _ => panic!("error: {err:?}"),
            },
        }
    }

    pub const fn is_registered(&self) -> bool {
        self.identifier.0 != u16::MAX
    }
}

#[derive(Debug)]
pub struct SourceNode {
    pub data: Source,
    pub children: Vec<SourceNode>,
}

impl SourceNode {
    pub fn new(data: Source) -> Self {
        Self {
            data,
            children: Vec::new(),
        }
    }
}
