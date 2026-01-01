use crate::core::diagnostic::{
    Diagnostic, DiagnosticList,
    reporter::reporter::{DiagnosticReportStatus, DiagnosticReporter},
};

#[derive(Debug)]
pub struct CLIReporter {
    pub diagnostics: DiagnosticList,
}

impl CLIReporter {
    pub fn new(diagnostics: DiagnosticList) -> Self {
        Self { diagnostics }
    }
}

impl DiagnosticReporter for CLIReporter {
    fn format_diagnostic(diagnostic: &Diagnostic) -> String {
        // TODO: Improve formatting
        let Diagnostic {
            span: _,
            severity,
            code,
            message,
            details: _,
        } = diagnostic;
        format!("{}<{}>: {}", severity, code, message)
    }

    fn report(&self) -> DiagnosticReportStatus {
        let mut status: DiagnosticReportStatus = [0; 3];

        self.diagnostics.iter().for_each(|diagnostic| {
            let formatted = CLIReporter::format_diagnostic(diagnostic);
            status[diagnostic.severity as usize] += 1_usize;

            println!("{formatted}");
        });

        status
    }
}
