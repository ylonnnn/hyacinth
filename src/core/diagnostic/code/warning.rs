use crate::core::diagnostic::code::code::DiagnosticCode;

#[repr(u32)]
#[derive(Debug)]
pub enum DiagnosticWarningKind {
    Unused,
}

impl DiagnosticCode {
    pub fn warning(kind: DiagnosticWarningKind) -> Self {
        Self::new(kind as u32)
    }
}
