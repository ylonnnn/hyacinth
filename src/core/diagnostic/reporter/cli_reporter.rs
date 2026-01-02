use crate::core::{
    Program,
    diagnostic::{
        Diagnostic,
        reporter::reporter::{DiagnosticReportStatus, DiagnosticReporter},
    },
};

#[derive(Debug)]
pub struct CLIReporter<'a> {
    pub program: &'a Program,
}

impl<'a> CLIReporter<'a> {
    pub fn new(program: &'a Program) -> Self {
        Self { program }
    }
}

impl<'a> DiagnosticReporter for CLIReporter<'a> {
    fn format_diagnostic(&self, diagnostic: &Diagnostic) -> String {
        // TODO: Improve formatting

        let Diagnostic {
            span,
            severity,
            code,
            message,
            details: _,
        } = diagnostic;
        let rc = span.to_rc(self.program);

        format!(
            "{}<{}>: {}\n{}:{}",
            severity, code, message, self.program.path, rc.0
        )
    }

    fn report(&self) -> DiagnosticReportStatus {
        let mut status: DiagnosticReportStatus = [0; 3];

        self.program
            .diagnostic_list()
            .data()
            .iter()
            .for_each(|diagnostic| {
                let formatted = self.format_diagnostic(diagnostic);
                status[diagnostic.severity as usize] += 1_usize;

                println!("{formatted}");
            });

        status
    }
}
