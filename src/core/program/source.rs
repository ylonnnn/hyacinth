use std::fs;

use crate::terminate;

#[derive(Debug, Clone)]
pub struct ProgramSource {
    pub identifier: Option<String>,
    pub data: String,
    pub lines: Vec<String>,
}

impl ProgramSource {
    pub fn new(identifier: String, data: String) -> Self {
        Self {
            identifier: Some(identifier),
            lines: data.lines().map(|s| s.to_string()).collect(),
            data,
        }
    }

    pub fn new_from_file(path: &str) -> Self {
        let Ok(data) = fs::read_to_string(path.to_string()) else {
            terminate!("hello, world");
        };

        Self::new(path.into(), data)
    }
}
