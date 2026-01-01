use crate::core::{
    Span,
    diagnostic::{
        code::DiagnosticCode,
        diagnostic::{Diagnostic, DiagnosticList, DiagnosticSeverity},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultStatus {
    Success,
    Fail,
}

#[derive(Debug)]
pub struct Result<T> {
    pub status: ResultStatus,
    pub data: Option<T>,
    pub diagnostics: DiagnosticList,
}

impl<T> Result<T> {
    pub fn adapt<U>(&mut self, other: &mut Result<U>) {
        if self.status != ResultStatus::Success {
            self.status = other.status;
        }

        self.diagnostics.append(&mut other.diagnostics);
    }

    pub fn consume(&mut self, mut other: Result<T>) {
        self.data = other.data.take();

        self.adapt(&mut other);
    }

    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) -> &Diagnostic {
        self.diagnostics.push(diagnostic);

        // Return latest diagnostic pushed
        self.diagnostics.iter().last().unwrap()
    }

    pub fn info(&mut self, code: DiagnosticCode, message: String, span: Span) -> &Diagnostic {
        self.add_diagnostic(Diagnostic::new(
            span,
            DiagnosticSeverity::Info,
            code,
            message,
        ))
    }

    pub fn warn(&mut self, code: DiagnosticCode, message: String, span: Span) -> &Diagnostic {
        self.add_diagnostic(Diagnostic::new(
            span,
            DiagnosticSeverity::Warning,
            code,
            message,
        ))
    }

    pub fn error(&mut self, code: DiagnosticCode, message: String, span: Span) -> &Diagnostic {
        self.add_diagnostic(Diagnostic::new(
            span,
            DiagnosticSeverity::Error,
            code,
            message,
        ))
    }
}

impl<T: Default> Default for Result<T> {
    fn default() -> Self {
        Self {
            data: Some(T::default()),
            status: ResultStatus::Success,
            diagnostics: Vec::new(),
        }
    }
}
