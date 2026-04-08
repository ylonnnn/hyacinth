use crate::diagnostic::Diagnostic;

pub trait DiagnosticReporter {
    fn format_diagnostic(&self, diagnostic: &Diagnostic, indentation: u8) -> String;
    fn report(&self);
}
