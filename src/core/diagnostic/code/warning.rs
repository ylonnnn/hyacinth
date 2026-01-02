use crate::core::diagnostic::code::code::DiagnosticCode;

#[repr(u32)]
#[derive(Debug)]
pub enum DiagnosticWarningKind {
    Unused = 325,
}

impl DiagnosticCode {
    pub fn warning(kind: DiagnosticWarningKind) -> Self {
        Self::new(kind as u32)
    }
}

impl From<DiagnosticWarningKind> for DiagnosticCode {
    fn from(value: DiagnosticWarningKind) -> Self {
        DiagnosticCode::warning(value)
    }
}
