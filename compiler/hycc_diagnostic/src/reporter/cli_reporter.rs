use crate::{
    Diagnostic, DiagnosticContext, DiagnosticCtx, DiagnosticSeverity,
    reporter::{DiagnosticReportStatus, DiagnosticReporter},
};

use hycc_source::{Source, SourceRegistry};
use hycc_span::Position;
use hycc_util::{Style, color, style, ternary};

#[derive(Debug)]
pub struct CLIReporter<'d, 's> {
    pub dctx: &'d DiagnosticCtx,
    pub source_registry: &'s SourceRegistry,
}

impl<'d, 's> CLIReporter<'d, 's> {
    pub fn new(dctx: &'d DiagnosticCtx, source_registry: &'s SourceRegistry) -> Self {
        Self {
            dctx,
            source_registry,
        }
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
        source: &Source,
        severity_color: &'static str,
        position_range: (Position, Position),
    ) -> String {
        let (start, end) = &position_range;
        let digit_n = ((start.line as f32).log10().floor() as usize) + 1;

        source
            .data
            .lines()
            .zip(1_u32..)
            .skip((start.line - 1) as usize)
            .take((end.line.saturating_sub(start.line) + 1) as usize)
            .map(|(line, num)| {
                let (bb, b, r) = (color::BRIGHT_BLUE, style::BOLD, style::RESET);
                let prefix = format!(
                    "  {line_num}  {pipe}  {r}",
                    line_num = num.to_string().style(bb),
                    pipe = "|".style(b),
                );

                let (ln_start, ln_end) = (
                    ternary!(num == start.line, start.column - 1, 0),
                    ternary!(num == end.line, end.column - 1, line.len() as u32),
                );

                format!("{prefix}{line}")
                    + &format!(
                        "\n{ptr_prefix}{padding}{pointer}",
                        ptr_prefix = format!(
                            "  {space}  {pipe}  {r}",
                            space = " ".repeat(digit_n as usize),
                            pipe = "|".style(b),
                        ),
                        padding = " ".repeat(ln_start as usize),
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
                    cs + &severity_color.bold(),
                    cs.reset().bright_white()
                )
            })
            .collect()
    }
}

impl<'d, 's> DiagnosticReporter for CLIReporter<'d, 's> {
    fn format_diagnostic(&self, diagnostic: &Diagnostic) -> String {
        let Diagnostic {
            span,
            severity,
            code,
            message,
            ..
        } = diagnostic;

        let Some(source) = self.source_registry.get(span.src_id) else {
            panic!("source identifier of span is invalid!");
        };

        let (start, end) = span.to_position_range(&source);
        let sev_color = self.color_from_severity(severity);

        format!(
            "{}<{}> -> {}{}\n{} {}:{}\n{}\n{}",
            severity.to_string().style(&sev_color).bold(),
            code.to_string().style(&sev_color),
            style::RESET,
            self.highlight(message, sev_color),
            "----->".bright_black(),
            source.identifier.1,
            start.clone(),
            self.emphasize(source, sev_color, (start, end)),
            style::RESET
        )
    }

    fn report(&self) -> DiagnosticReportStatus {
        let mut status: DiagnosticReportStatus = [0; 3];

        for diagnostic in self.dctx.data() {
            let formatted = self.format_diagnostic(diagnostic);
            status[diagnostic.severity as usize] += 1;

            println!("{formatted}");
        }

        status
    }
}
