use crate::core::diagnostic::code::code::DiagnosticCode;

#[repr(u32)]
#[derive(Debug)]
pub enum DiagnosticErrorKind {
    InvalidNumericLiteralPrefix,
}

impl DiagnosticCode {
    pub fn error(kind: DiagnosticErrorKind) -> Self {
        Self::new(kind as u32)
    }
}
