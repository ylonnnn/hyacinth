use std::fmt::{self, Display};

use crate::core::{Span, diagnostic::code::DiagnosticCode};

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

    pub fn detail(
        &mut self,
        span: Span,
        severity: DiagnosticSeverity,
        code: DiagnosticCode,
        message: String,
    ) -> &mut Self {
        self.details
            .push(Box::new(Diagnostic::new(span, severity, code, message)));

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
pub struct DiagnosticList(Vec<Diagnostic>);

impl DiagnosticList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn data(&self) -> &Vec<Diagnostic> {
        &self.0
    }

    pub fn data_mut(&mut self) -> &mut Vec<Diagnostic> {
        &mut self.0
    }

    pub fn add(&mut self, diagnostic: Diagnostic) -> &mut Diagnostic {
        self.0.push(diagnostic);
        self.0.iter_mut().last().unwrap()
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

impl Default for DiagnosticList {
    fn default() -> Self {
        Self(Vec::with_capacity(32))
    }
}
