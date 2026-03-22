use std::fmt::{self, Display};

use crate::code::DiagnosticCode;
use hycc_span::Span;

#[derive(Debug)]
pub struct Diagnostic {
    pub span: Span,
    pub severity: DiagnosticSeverity,
    pub code: DiagnosticCode,
    pub message: String,
    pub details: Vec<Box<Diagnostic>>,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

impl Diagnostic {
    pub fn new(
        span: Span,
        severity: DiagnosticSeverity,
        code: DiagnosticCode,
        message: String,
    ) -> Self {
        Self {
            span,
            severity,
            code,
            message,
            details: Vec::new(),
        }
    }

    pub fn add_detail(&mut self, detail: Diagnostic) -> &mut Self {
        self.details.push(Box::new(detail));
        self
    }

    pub fn detail(
        &mut self,
        span: Span,
        severity: DiagnosticSeverity,
        code: DiagnosticCode,
        message: String,
    ) -> &mut Self {
        self.add_detail(Diagnostic::new(span, severity, code, message));
        self
    }
}

impl Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Self::Info => "info",
                Self::Warning => "warning",
                Self::Error => "error",
            }
        )
    }
}

#[derive(Debug)]
pub struct DiagnosticContext {
    pub data: Vec<Diagnostic>,
}

impl DiagnosticContext {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn add(&mut self, diagnostic: Diagnostic) -> &mut Diagnostic {
        self.data.push(diagnostic);
        self.data.last_mut().unwrap()
    }

    pub fn info(&mut self, code: DiagnosticCode, message: &str, span: Span) -> &Diagnostic {
        self.add(Diagnostic::new(
            span,
            DiagnosticSeverity::Info,
            code,
            message.into(),
        ))
    }

    pub fn warn(&mut self, code: DiagnosticCode, message: &str, span: Span) -> &Diagnostic {
        self.add(Diagnostic::new(
            span,
            DiagnosticSeverity::Warning,
            code,
            message.into(),
        ))
    }

    pub fn error(&mut self, code: DiagnosticCode, message: &str, span: Span) -> &Diagnostic {
        self.add(Diagnostic::new(
            span,
            DiagnosticSeverity::Error,
            code,
            message.into(),
        ))
    }
}

impl Default for DiagnosticContext {
    fn default() -> Self {
        Self {
            data: Vec::with_capacity(32),
        }
    }
}
