use std::fmt::{self, Display};

use hycc_span::Span;

use crate::code::DiagnosticCode;

#[derive(Debug)]
pub struct Diagnostic {
    pub span: Span,
    pub severity: DiagnosticSeverity,
    pub code: DiagnosticCode,
    pub message: String,
    pub details: Vec<Box<Diagnostic>>,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub fn is_info(&self) -> bool {
        self.severity == DiagnosticSeverity::Info
    }

    pub fn is_warning(&self) -> bool {
        self.severity == DiagnosticSeverity::Warning
    }

    pub fn is_error(&self) -> bool {
        self.severity == DiagnosticSeverity::Error
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

pub trait DiagnosticContext {
    fn data(&self) -> &Vec<Diagnostic>;
    fn data_mut(&mut self) -> &mut Vec<Diagnostic>;

    fn error_occurred(&self) -> bool;

    fn add(&mut self, diagnostic: Diagnostic) -> Option<&mut Diagnostic> {
        let data = self.data_mut();

        data.push(diagnostic);
        data.last_mut()
    }

    fn info(&mut self, code: DiagnosticCode, message: &str, span: Span) -> &Diagnostic {
        self.add(Diagnostic::new(
            span,
            DiagnosticSeverity::Info,
            code,
            message.into(),
        ))
        .unwrap()
    }

    fn warn(&mut self, code: DiagnosticCode, message: &str, span: Span) -> &Diagnostic {
        self.add(Diagnostic::new(
            span,
            DiagnosticSeverity::Warning,
            code,
            message.into(),
        ))
        .unwrap()
    }

    fn error(&mut self, code: DiagnosticCode, message: &str, span: Span) -> Option<&Diagnostic> {
        Some(
            self.add(Diagnostic::new(
                span,
                DiagnosticSeverity::Error,
                code,
                message.into(),
            ))
            .unwrap(),
        )
    }
}

#[derive(Debug)]
pub struct DiagnosticCtx(Vec<Diagnostic>, bool);

impl Default for DiagnosticCtx {
    fn default() -> Self {
        Self(Vec::with_capacity(32), false)
    }
}

impl DiagnosticCtx {
    pub fn new() -> Self {
        Self(Vec::new(), false)
    }
}

impl DiagnosticContext for DiagnosticCtx {
    fn data(&self) -> &Vec<Diagnostic> {
        &self.0
    }

    fn data_mut(&mut self) -> &mut Vec<Diagnostic> {
        &mut self.0
    }

    fn add(&mut self, diagnostic: Diagnostic) -> Option<&mut Diagnostic> {
        if diagnostic.severity == DiagnosticSeverity::Error {
            self.1 = true
        }

        let data = self.data_mut();

        data.push(diagnostic);
        data.last_mut()
    }

    fn error_occurred(&self) -> bool {
        self.1
    }
}
