use crate::{
    Diagnostic, DiagnosticContext, DiagnosticCtx, diagnostic::DiagnosticKind,
    reporter::DiagnosticReporter,
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
    ) -> Vec<String> {
        let (start, end) = &position_range;
        let digit_n = ((end.line as f32).log10().floor() as usize) + 1;

        source
            .data
            .lines()
            .zip(1_u32..)
            .skip((start.line - 1) as usize)
            .take((end.line.saturating_sub(start.line) + 1) as usize)
            .flat_map(|(line, num)| {
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

                [
                    format!("{prefix}{line}"),
                    format!(
                        "{ptr_prefix}{padding}{pointer}",
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
                    ),
                ]
            })
            .collect()
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
    fn format_diagnostic(&self, diagnostic: &Diagnostic, indentation: u8) -> String {
        let Diagnostic {
            span,
            kind,
            details,
            ..
        } = diagnostic;

        let source = self.source_registry.get(span.src_id);
        let (s_kind, code, message) = kind.data();

        let (start, end) = span.to_position_range(&source);
        let sev_color = self.color(&kind);

        let indentation = 6 * indentation as usize;
        let s_kind_indent = "=".bright_black().repeat(indentation.saturating_sub(2));
        let indent = " ".repeat(indentation);

        let Ok(cwd) = std::env::current_dir() else {
            panic!("failed to retrieve the current directory")
        };

        format!(
            " {s_kind_indent} {}{} {reset}{}\n{indent}{} {}:{}\n{indent}{emphasis}\n{indent}{reset}\n{details}",
            s_kind.to_string().style(&sev_color).bold(),
            code.map_or_else(
                || ":".into(),
                |code| format!("<{}>", code.to_string().style(&sev_color))
            ),
            self.highlight(message, sev_color),
            "----->".bright_black(),
            &source.identifier.1.replace(cwd.to_str().unwrap(), "")[1..],
            start.clone(),
            emphasis = self
                .emphasize(source, sev_color, (start, end))
                .join(&format!("\n{indent}")),
            reset = style::RESET,
            details = details
                .iter()
                .map(|diag| self.format_diagnostic(diag, indentation as u8 + 1))
                .collect::<Vec<String>>()
                .join("\n\n")
        )
    }

    fn report(&self) {
        for diagnostic in self.dctx.data() {
            println!("{}", self.format_diagnostic(diagnostic, 0));
        }
    }
}
