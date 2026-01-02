use crate::core::diagnostic::code::code::DiagnosticCode;

#[repr(u32)]
#[derive(Debug)]
pub enum DiagnosticInfoKind {
    Note = 250,
    Suggestion,
}

impl DiagnosticCode {
    pub fn info(kind: DiagnosticInfoKind) -> Self {
        Self::new(kind as u32)
    }
}

impl From<DiagnosticInfoKind> for DiagnosticCode {
    fn from(value: DiagnosticInfoKind) -> Self {
        DiagnosticCode::info(value)
    }
}
