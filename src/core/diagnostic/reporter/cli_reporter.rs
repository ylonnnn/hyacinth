use crate::{
    core::{
        Position,
        diagnostic::{
            Diagnostic, DiagnosticSeverity,
            reporter::reporter::{DiagnosticReportStatus, DiagnosticReporter},
        },
        program::Program,
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

    pub fn color_from_severity(&self, severity: &DiagnosticSeverity) -> &'static str {
        match severity {
            DiagnosticSeverity::Info => color::BRIGHT_BLUE,
            DiagnosticSeverity::Warning => color::YELLOW,
            DiagnosticSeverity::Error => color::RED,
        }
    }

    pub fn emphasize(
        &self,
        severity_color: &'static str,
        position_range: (Position, Position),
    ) -> String {
        let (start, end) = &position_range;
        self.program
            .lexer
            .source
            .lines
            .iter()
            .zip(1_u32..)
            .skip((start.line - 1) as usize)
            .take((end.line.saturating_sub(start.line) + 1) as usize)
            .map(|(line, num)| {
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
                    ternary!(num == end.line, end.column - 1, line.len() as u32),
                );

                format!("{prefix}{line}")
                    + &format!(
                        "\n{padding}{pointer}",
                        padding = " ".repeat(p_len + ln_start as usize),
                        pointer = "^"
                            .repeat((ln_end - ln_start) as usize)
                            .style(severity_color)
                            .bold()
                    )
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn highlight(&self, message: &String, severity_color: &'static str) -> String {
        let mut highlight = false;
        message
            .chars()
            .map(|c| {
                let cs = c.to_string();
                if c != '`' {
                    return cs;
                }

                highlight = !highlight;
                ternary!(
                    highlight,
                    (cs + severity_color).bold(),
                    cs.reset().bright_white()
                )
            })
            .collect()
    }
}

impl<'a> DiagnosticReporter for CLIReporter<'a> {
    fn format_diagnostic(&self, diagnostic: &Diagnostic) -> String {
        let Diagnostic {
            span,
            severity,
            code,
            message,
            details: _,
        } = diagnostic;

        let source = &self.program.source;
        let (start, end) = span.to_position_range(&source);
        let sev_color = self.color_from_severity(severity);

        format!(
            "{}<{}> -> {}{}\n{} {}:{}\n{}\n{}",
            severity.to_string().style(&sev_color).bold(),
            code.to_string().style(&sev_color),
            style::RESET,
            self.highlight(message, sev_color),
            "----->".bright_black(),
            ternary!(
                source.identifier.is_some(),
                source.identifier.as_ref().unwrap(),
                "no-identifier.hyc"
            ),
            start.clone(),
            self.emphasize(sev_color, (start, end)),
            style::RESET
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
