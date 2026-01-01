use crate::core::diagnostic::Diagnostic;

pub type DiagnosticReportStatus = [usize; 3];

pub trait DiagnosticReporter {
    fn format_diagnostic(diagnostic: &Diagnostic) -> String;
    fn report(&self) -> DiagnosticReportStatus;
}
