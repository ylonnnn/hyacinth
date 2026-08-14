use crate::diagnostic::Diag;

pub trait DiagnosticReporter {
    fn format_diagnostic(&self, diagnostic: &Diag, indentation: u8) -> String;
    fn report(&self);
}
