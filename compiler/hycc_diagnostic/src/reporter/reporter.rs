use crate::diagnostic::Diag;

pub trait DiagnosticReporter {
    fn format_diagnostic(&self, diagnostic: &Diag) -> String;
    fn report(&self);
}
