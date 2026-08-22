use std::collections::{HashMap, HashSet};

use crate::{
    diagnostic::{Diag, DiagCtx, DiagDetail, DiagDetailKind, DiagKind},
    reporter::reporter::DiagnosticReporter,
};

use hycc_source::{Source, SourceRegistry};
use hycc_span::{Position, Span};
use hycc_util::{Style, color, style, ternary};

#[derive(Debug)]
struct EmphasisData {
    pub messages: Vec<(u32, (&'static str, String))>,
    pub lines: Vec<EmphasisLine>,
}

#[derive(Debug)]
struct EmphasisLine {
    pub ptrs: Vec<EmphasisPointer>,
    pub content: String,
    pub line_no: u32,
}

#[derive(Debug)]
struct EmphasisPointer {
    pub len: usize,
    pub offset: u32,
    pub display: (&'static str, char),
}

#[derive(Debug)]
struct EmphasisTarget {
    pub display: (&'static str, char),
    pub message: String,
    pub span: Span,
}

impl EmphasisTarget {
    fn new(message: String, span: Span, display: (&'static str, char)) -> Self {
        Self {
            message,
            span,
            display,
        }
    }
}

#[derive(Debug)]
pub struct CLIReporter<'d, 's> {
    pub dctx: &'d DiagCtx,
    pub source_registry: &'s SourceRegistry,
}

impl<'d, 's> CLIReporter<'d, 's> {
    pub fn new(dctx: &'d DiagCtx, source_registry: &'s SourceRegistry) -> Self {
        Self {
            dctx,
            source_registry,
        }
    }

    pub fn color(&self, kind: &DiagKind) -> &'static str {
        match kind {
            DiagKind::Info => color::BRIGHT_BLUE,
            DiagKind::Warning => color::YELLOW,
            DiagKind::Error(..) => color::RED,
        }
    }

    pub fn detail_color(&self, kind: &DiagDetailKind) -> &'static str {
        match kind {
            DiagDetailKind::Note => color::BRIGHT_BLUE,
            DiagDetailKind::Help => color::BRIGHT_CYAN,
        }
    }

    fn emphasize(&self, source: &Source, mut targets: Vec<EmphasisTarget>) -> Vec<EmphasisData> {
        // Group overlapping targets
        let mut g_targets = Vec::new();
        while !targets.is_empty() {
            let mut group = vec![targets.swap_remove(0)];
            let mut j = 0;

            let (b_start, b_end) = group[0].span.to_position_range(&source);
            while j < targets.len() {
                let (start, end) = targets[j].span.to_position_range(&source);
                if start.line >= b_start.line && end.line <= b_end.line {
                    // if group[0].span.overlaps(targets[j].span) {
                    group.push(targets.swap_remove(j));
                    continue;
                }

                j += 1;
            }

            g_targets.push(group);
        }

        let mut data = g_targets
            .into_iter()
            .map(|mut group| {
                group.sort_by(|a, b| {
                    b.span.offset.cmp(&a.span.offset)
                    // .then_with(|| b.span.len.cmp(&a.span.len))
                });

                let mut lines = HashMap::<u32, EmphasisLine>::new();
                let mut messages = Vec::new();

                for target in group {
                    let (color, ptr) = target.display;
                    let (start, end) = target.span.to_position_range(&source);
                    let digit_n = ((end.line as f32).log10().floor() as usize) + 1;
                    let n = end.line - start.line;

                    messages.push((
                        end.column - 1, // Inclusion
                        (color, target.message.clone()),
                    ));

                    source
                        .data
                        .lines()
                        .zip(1_u32..)
                        .skip((start.line - 1) as usize)
                        .take((end.line.saturating_sub(start.line) + 1) as usize)
                        // .enumerate()
                        // .filter_map(|(i, pair)| {
                        //     ternary!(
                        //         n <= 5 || (i <= 2 || (i >= end.line as usize - 2 && i < end.line as usize)),
                        //         Some(pair),
                        //         None
                        //     )
                        // })
                        .for_each(|(line, num)| {
                            let (bb, b, r) = (color::BRIGHT_BLUE, style::BOLD, style::RESET);
                            let dig_n = ((num as f32).log10().floor() as usize) + 1;

                            // let prefix = format!(
                            //     " {}{}| {r}",
                            //     num.to_string().style(bb).style(b),
                            //     " ".repeat((1 + digit_n) - dig_n),
                            // );

                            let (ln_start, ln_end) = (
                                ternary!(num == start.line, start.column - 1, 0),
                                ternary!(num == end.line, end.column - 1, line.len() as u32),
                            );

                            if let Some(line) = lines.get_mut(&num) {
                                line.ptrs.push(EmphasisPointer {
                                    len: ((ln_end - ln_start) as usize).clamp(1, usize::MAX),
                                    offset: ln_start,
                                    display: (color, ptr),
                                })
                            } else {
                                lines.insert(
                                    num,
                                    EmphasisLine {
                                        line_no: num,
                                        content: String::from(line),
                                        ptrs: vec![EmphasisPointer {
                                            len: ((ln_end - ln_start) as usize)
                                                .clamp(1, usize::MAX),
                                            offset: ln_start,
                                            display: (color, ptr),
                                        }],
                                    },
                                );
                            }
                        })
                }

                let mut lines = lines.into_iter().map(|(_, val)| val).collect::<Vec<_>>();
                lines.sort_by_key(|line| line.line_no);

                EmphasisData { lines, messages }
            })
            .collect::<Vec<_>>();

        // TODO: fix diagnostic emphasis ordering
        // data.sort_by(|a, b| a.lines[0].line_no.cmp(&b.lines[0].line_no));
        data

        // // let n = emphasized.len();
        // // let mid = n / 2;
        //
        // ternary!(
        //     (n / 2) < 3,
        //     emphasized,
        //     emphasized
        //         .into_iter()
        //         .enumerate()
        //         .filter(|(i, _)| *i != mid)
        //         .map(|(i, line)| ternary!(i == mid + 1, "  ...".bright_blue().bold(), line))
        //         .collect()
        // )
    }

    pub fn highlight(&self, message: &String, severity_color: &'static str) -> String {
        let mut highlight = false;
        message
            .chars()
            .map(|c| {
                let cs = c.to_string();
                ternary!(c != '`', cs, {
                    highlight = !highlight;
                    ternary!(
                        highlight,
                        cs + &severity_color.bold(),
                        cs.reset().bright_white()
                    )
                })
            })
            .collect()
    }

    pub fn annotate_snippet(&self, diag: &Diag) -> Vec<String> {
        let source = self.source_registry.get(diag.span.src_id);
        let (start, end) = diag.span.to_position_range(&source);

        let mut data = self.emphasize(
            &source,
            std::iter::once(EmphasisTarget::new(
                diag.message
                    .1
                    .as_ref()
                    .map_or_else(|| "".into(), |extra| extra.clone()),
                diag.span,
                (self.color(&diag.kind), '^'),
            ))
            .chain(diag.details.iter().map(|detail| {
                let ptr = match &detail.kind {
                    DiagDetailKind::Note => '-',
                    DiagDetailKind::Help => '~',
                };

                EmphasisTarget::new(
                    detail.message.clone(),
                    detail.span,
                    (self.detail_color(&detail.kind), ptr),
                )
            }))
            .collect(),
        );

        let max_lno = data
            .iter()
            .flat_map(|data| &data.lines)
            .map(|line| line.line_no)
            .max()
            .unwrap();
        let pref_fn = |line_no: Option<u32>| {
            let digits = line_no.map_or_else(|| 0, |line| (line as f32).log10() as usize + 1);

            format!(
                "  {}{}  {}{}  ",
                line_no
                    .map_or_else(|| "".into(), |line| line.to_string())
                    .style(color::BRIGHT_BLUE)
                    .bold(),
                " ".repeat((((max_lno as f32).log10() as usize) + 1) - digits),
                "|",
                style::RESET,
            )
        };

        data.into_iter()
            .map(|data| {
                let n = data.lines.last().unwrap().content.len();
                let formatted = data
                    .lines
                    .into_iter()
                    .map(move |mut line| {
                        let n = line.content.len();

                        line.ptrs.sort_by(|a_ptr, b_ptr| {
                            a_ptr
                                .offset
                                .cmp(&b_ptr.offset)
                                .then_with(|| b_ptr.len.cmp(&a_ptr.len))
                        });

                        let mut ptrs = vec![None; n + 1];
                        for (i, ptr) in line.ptrs.iter().enumerate() {
                            let &EmphasisPointer { offset, len, .. } = &ptr;
                            let offset = *offset as usize;
                            let (_, ptr_char) = &ptr.display;

                            (offset..(offset + len)).for_each(|idx| ptrs[idx] = Some(i));
                        }

                        let mut prev = None;
                        let mut pointer = ptrs
                            .into_iter()
                            .map(|ptr_idx| {
                                let Some(idx) = ptr_idx else {
                                    return " ".into();
                                };

                                let ptr = &line.ptrs[idx];
                                if let Some(prev_idx) = prev
                                    && idx == prev_idx
                                {
                                    return ptr.display.1.to_string();
                                }

                                ptr.display.1.to_string().style(ptr.display.0)
                            })
                            .chain(std::iter::once(style::RESET.into()))
                            .collect::<String>();

                        format!(
                            "{}{}\n{}{}",
                            pref_fn(Some(line.line_no)),
                            line.content,
                            pref_fn(None),
                            pointer.bold(),
                        )
                    })
                    .collect::<Vec<_>>();

                let messages = data
                    .messages
                    .iter()
                    .rev()
                    .enumerate()
                    .map(|(i, (_, (_, basis)))| {
                        let mut line = " ".repeat(n);
                        data.messages.iter().enumerate().skip(i).for_each(
                            |(j, (offset, (color, message)))| {
                                let offset = *offset as usize - 1;
                                let idx = line
                                    .char_indices()
                                    .nth(n)
                                    .map(|(idx, _)| idx)
                                    .unwrap_or(line.len());

                                ternary!(
                                    i == j,
                                    line.insert_str(idx, &message.style(color).bold(),),
                                    {
                                        if line.chars().nth(idx).is_some_and(char::is_whitespace) {
                                            let bar = "|".style(color).bold();
                                            line.replace_range(idx..(idx + 1), &bar);
                                        }
                                    }
                                )
                            },
                        );

                        format!("{}{}", pref_fn(None), line)
                    })
                    .collect::<Vec<_>>();

                format!("{}\n{}", formatted.join("\n"), messages.join("\n"))
            })
            .collect()
    }
}

impl<'d, 's> DiagnosticReporter for CLIReporter<'d, 's> {
    fn format_diagnostic(&self, diag: &Diag) -> String {
        let source = self.source_registry.get(diag.span.src_id);
        let (s_kind, code) = diag.kind.data();

        let cwd = std::env::current_dir()
            .unwrap_or_else(|_| panic!("failed to retrieve the current directory"));

        let snippet = self.annotate_snippet(&diag).join("\n");
        let sub_diagnostics = diag
            .sub_diagnostics
            .iter()
            .map(|sub| self.format_diagnostic(&sub))
            .collect::<String>();

        let (start, end) = diag.span.to_position_range(&source);
        let sev_color = self.color(&diag.kind);

        format!(
            "{}{} {reset}{}\n{} {}:{}\n{snippet}\n{reset}{sub_diagnostics}",
            s_kind.to_string().style(&sev_color).bold(),
            code.map_or_else(|| ":".into(), |code| format!("<E{:04}>", code)),
            self.highlight(&diag.message.0, sev_color),
            "----->".bright_black(),
            &source.identifier.1.replace(cwd.to_str().unwrap(), "")[1..],
            start.clone(),
            reset = style::RESET
        )
    }

    fn report(&self) {
        for diagnostic in self.dctx.data() {
            println!("{}", self.format_diagnostic(diagnostic));
        }
    }
}
