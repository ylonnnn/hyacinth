use crate::core::diagnostic::code::code::DiagnosticCode;

#[repr(u32)]
#[derive(Debug)]
pub enum DiagnosticInfoKind {
    Note,
    Suggestion,
}

impl DiagnosticCode {
    pub fn info(kind: DiagnosticInfoKind) -> Self {
        Self::new(kind as u32)
    }
}
