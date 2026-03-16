use crate::core::diagnostic::code::code::DiagnosticCode;

#[repr(u32)]
#[derive(Debug)]
pub enum DiagnosticErrorKind {
    UnknownCharacter,
    InvalidNumericLiteralPrefix = 400,
    InvalidNumericLiteralDigit,
    UnterminatedCharacterSequence,
    InvalidCharacterSequence,

    UnexpectedToken,
    MissingExplicitTypeAnnotation,
    InvalidVariableDeclaration,
}

impl DiagnosticCode {
    pub fn error(kind: DiagnosticErrorKind) -> Self {
        Self::new(kind as u32)
    }
}

impl From<DiagnosticErrorKind> for DiagnosticCode {
    fn from(value: DiagnosticErrorKind) -> Self {
        DiagnosticCode::error(value)
    }
}
