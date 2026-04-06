use crate::{
    Diagnostic, DiagnosticContext, DiagnosticCtx,
    diagnostic::DiagnosticKind,
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

    pub fn color(&self, kind: &DiagnosticKind) -> &'static str {
        match kind {
            DiagnosticKind::Note(..) => color::BRIGHT_BLUE,
            DiagnosticKind::Warning(..) => color::YELLOW,
            DiagnosticKind::Error(..) => color::RED,
        }
    }

    pub fn emphasize(
        &self,
        source: &Source,
        severity_color: &'static str,
        position_range: (Position, Position),
    ) -> String {
        let (start, end) = &position_range;
        let digit_n = ((end.line as f32).log10().floor() as usize) + 1;

        source
            .data
            .lines()
            .zip(1_u32..)
            .skip((start.line - 1) as usize)
            .take((end.line.saturating_sub(start.line) + 1) as usize)
            .map(|(line, num)| {
                let (bb, b, r) = (color::BRIGHT_BLUE, style::BOLD, style::RESET);
                let dig_n = ((num as f32).log10().floor() as usize) + 1;

                let prefix = format!(
                    "  {line_num}{padding}{pipe}  {r}",
                    padding = " ".repeat((2 + digit_n) - dig_n),
                    line_num = num.to_string().style(bb).style(b),
                    pipe = "|",
                );

                let (ln_start, ln_end) = (
                    ternary!(num == start.line, start.column - 1, 0),
                    ternary!(num == end.line, end.column - 1, line.len() as u32),
                );

                format!(
                    "{prefix}{line}\n{ptr_prefix}{padding}{pointer}",
                    ptr_prefix = format!(
                        "  {space}  {pipe}  {r}",
                        space = " ".repeat(digit_n as usize),
                        pipe = "|".style(bb).style(b),
                    ),
                    padding = " ".repeat(ln_start as usize),
                    pointer = "^"
                        .repeat(((ln_end - ln_start) as usize).clamp(1, usize::MAX))
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
        let Diagnostic { span, kind, .. } = diagnostic;

        let source = self.source_registry.get(span.src_id);
        let (s_kind, code, args) = kind.data();

        let (start, end) = span.to_position_range(&source);
        let sev_color = self.color(&kind);

        format!(
            "{}<{}> -> {}{}\n{} {}:{}\n{}\n{}",
            s_kind.to_string().style(&sev_color).bold(),
            code.to_string().style(&sev_color),
            style::RESET,
            self.highlight(&format!("{}", args), sev_color),
            "----->".bright_black(),
            source.identifier.1,
            start.clone(),
            self.emphasize(source, sev_color, (start, end)),
            style::RESET
        )
    }

    fn report(&self) -> DiagnosticReportStatus {
        let status: DiagnosticReportStatus = [0; 3];

        for diagnostic in self.dctx.data() {
            let formatted = self.format_diagnostic(diagnostic);
            // status[diagnostic.severity as usize] += 1;

            println!("{formatted}");
        }

        status
    }
}
