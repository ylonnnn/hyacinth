use crate::{
    core::{
        Program,
        diagnostic::{
            Diagnostic, DiagnosticSeverity,
            reporter::reporter::{DiagnosticReportStatus, DiagnosticReporter},
        },
    },
    ternary,
    utils::{Style, color, style},
};

#[derive(Debug)]
pub struct CLIReporter<'a> {
    pub program: &'a mut Program,
}

impl<'a> CLIReporter<'a> {
    pub fn new(program: &'a mut Program) -> Self {
        Self { program }
    }

    pub fn color_from_severity(severity: &DiagnosticSeverity) -> &'static str {
        match severity {
            DiagnosticSeverity::Info => color::BRIGHT_BLUE,
            DiagnosticSeverity::Warning => color::YELLOW,
            DiagnosticSeverity::Error => color::RED,
        }
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

        let (start, end) = span.to_rc(self.program);
        let sev_color = CLIReporter::color_from_severity(severity);

        // Slices Lines
        let lines = &mut self
            .program
            .lexer
            .source
            .lines()
            .zip(1_usize..)
            .skip(start.line - 1)
            .take(end.line.saturating_sub(start.line) + 1)
            .map(|(line, num)| {
                let len = line.len();

                let (bb, b, r) = (color::BRIGHT_BLUE, style::BOLD, style::RESET);
                let prefix = format!(
                    "  {line_num}  {pipe}  {reset}",
                    line_num = num.to_string().style(bb),
                    pipe = "|".style(b),
                    reset = r
                );

                let p_len = prefix.len() - (bb.len() + b.len() + r.len());
                let (ln_start, ln_end) = (
                    ternary!(num == start.line, start.column - 1, 0),
                    ternary!(num == end.line, end.column - 1, len),
                );

                format!("{prefix}{line}")
                    + &format!(
                        "\n{padding}{pointer}",
                        padding = " ".repeat(p_len + ln_start),
                        pointer = "^".repeat(ln_end - ln_start).style(sev_color).bold()
                    )
            })
            .collect::<Vec<String>>();

        let mut t_s = false;
        let message: String = message
            .chars()
            .map(|c| {
                let cs = c.to_string();
                if c != '`' {
                    return cs;
                }

                t_s = !t_s;
                ternary!(t_s, cs + sev_color, cs.bright_white())
            })
            .collect();

        format!(
            "{}<{}>: {}\n{} {}:{}\n{}\n",
            "".bold() + &severity.to_string().style(&sev_color),
            code.to_string().style(&sev_color),
            message.bright_white(),
            "found at:".blue(),
            self.program.path.reset(),
            start,
            lines.join("\n")
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
