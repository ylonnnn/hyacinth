use std::fmt::Display;

pub use crate::core::diagnostic::code::{error::*, info::*, warning::*};

#[derive(Debug)]
pub struct DiagnosticCode(u32);

impl DiagnosticCode {
    pub fn new(code: u32) -> Self {
        Self(code)
    }
}

impl Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0{}", self.0)
    }
}
