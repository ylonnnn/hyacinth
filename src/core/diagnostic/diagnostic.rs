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

pub type DiagnosticList = Vec<Diagnostic>;
