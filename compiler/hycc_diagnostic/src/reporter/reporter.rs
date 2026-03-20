use crate::diagnostic::Diagnostic;

pub type DiagnosticReportStatus = [usize; 3];

pub trait DiagnosticReporter {
    fn format_diagnostic(&self, diagnostic: &Diagnostic) -> String;
    fn report(&self) -> DiagnosticReportStatus;
}
