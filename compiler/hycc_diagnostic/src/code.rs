use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct DiagnosticCode(pub u16);

impl Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}", self.0)
    }
}

// #[repr(u32)]
// #[derive(Debug)]
// pub enum DiagnosticInfoKind {
//     Note = 250,
//     Suggestion,
// }

// impl DiagnosticCode {
//     pub fn info(kind: DiagnosticInfoKind) -> Self {
//         Self::new(kind as u32)
//     }
// }

// impl From<DiagnosticInfoKind> for DiagnosticCode {
//     fn from(value: DiagnosticInfoKind) -> Self {
//         DiagnosticCode::info(value)
//     }
// }

// #[repr(u32)]
// #[derive(Debug)]
// pub enum DiagnosticWarningKind {
//     Unused = 325,
// }

// impl DiagnosticCode {
//     pub fn warning(kind: DiagnosticWarningKind) -> Self {
//         Self::new(kind as u32)
//     }
// }

// impl From<DiagnosticWarningKind> for DiagnosticCode {
//     fn from(value: DiagnosticWarningKind) -> Self {
//         DiagnosticCode::warning(value)
//     }
// }

// #[repr(u32)]
// #[derive(Debug)]
// pub enum DiagnosticErrorKind {
//     UnknownCharacter = 400,
//     InvalidNumericLiteralprefix,
//     InvalidNumericLiteralDigit,
//     UnterminatedCharacterSequence,
//     InvalidCharacterSequence,

//     UnclosedDelimeterCollection,

//     UnexpectedToken,
//     InvalidVariableDeclaration,
// }

// impl DiagnosticCode {
//     pub fn error(kind: DiagnosticErrorKind) -> Self {
//         Self::new(kind as u32)
//     }
// }

// impl From<DiagnosticErrorKind> for DiagnosticCode {
//     fn from(value: DiagnosticErrorKind) -> Self {
//         DiagnosticCode::error(value)
//     }
// }
