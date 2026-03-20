use std::fs;

use hycc_util::terminate;

#[derive(Debug, Clone)]
pub struct Source {
    pub identifier: (u16, String),
    pub data: String,
}

impl Source {
    pub fn new(identifier: String, data: String) -> Self {
        Self {
            identifier: (u16::MAX, identifier),
            data,
        }
    }

    pub fn new_from_file(path: &str) -> Self {
        match fs::read_to_string(path.to_string()) {
            Ok(data) => Self::new(path.into(), data),
            Err(err) => match err {
                _ => panic!("error: {err:?}"),
            },
        }
    }

    pub const fn is_registered(&self) -> bool {
        self.identifier.0 != u16::MAX
    }
}
