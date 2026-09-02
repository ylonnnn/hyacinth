use std::collections::{HashMap, HashSet, hash_map::Entry};

use crate::{
    diagnostic::{Diag, DiagCtx, DiagDetail, DiagDetailKind, DiagKind},
    reporter::reporter::DiagnosticReporter,
};

use hycc_source::{Source, SourceRegistry, source::SourceId};
use hycc_span::{Position, Span};
use hycc_util::{Style, color, style, ternary};

#[derive(Debug)]
struct EmphasisData {
    pub messages: HashMap<u32, Vec<EmphasisMessage>>,
    pub lines: Vec<EmphasisLine>,
    pub span: Span,
}

#[derive(Debug)]
struct EmphasisMessage {
    pub content: String,
    pub color: &'static str,
    pub offset: u32,
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
    pub message: String,
    pub display: (&'static str, char),
    pub span: Span,
}

impl EmphasisTarget {
    fn new(message: String, span: Span, display: (&'static str, char)) -> Self {
        Self {
            message,
            display,
            span,
        }
    }
}

#[derive(Debug)]
pub struct CLIReporter<'r> {
    pub dctx: &'r DiagCtx,
    pub source_registry: &'r SourceRegistry,
}

impl<'r> CLIReporter<'r> {
    pub fn new(dctx: &'r DiagCtx, source_registry: &'r SourceRegistry) -> Self {
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
            DiagDetailKind::Primary(kind) => self.color(&kind),
            DiagDetailKind::Note => color::BRIGHT_BLUE,
            DiagDetailKind::Help => color::BRIGHT_GREEN,
        }
    }

    fn emphasize(&self, targets: Vec<EmphasisTarget>) -> Vec<EmphasisData> {
        let g_targets = targets
            .into_iter()
            .fold(HashMap::<_, Vec<_>>::new(), |mut map, target| {
                map.entry(target.span.src_id).or_default().push(target);
                map
            });

        g_targets
            .into_iter()
            .map(|(src_id, group)| {
                let group_span = group[0].span;

                let mut lines = HashMap::<u32, EmphasisLine>::new();
                let mut messages = HashMap::<u32, Vec<EmphasisMessage>>::new();

                for target in group {
                    let source = self.source_registry.get(src_id);
                    let ((color, ptr), (start, end)) =
                        (target.display, target.span.to_position_range(&source));
                    let (digit_n, n) = (
                        ((end.line as f32).log10().floor() as usize) + 1,
                        end.line - start.line,
                    );

                    messages.entry(end.line).or_default().push(EmphasisMessage {
                        content: target.message,
                        color,
                        offset: end.column - 1, // Inclusion
                    });

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

                EmphasisData {
                    span: group_span,
                    lines,
                    messages,
                }
            })
            .collect()
    }

    pub fn highlight(&self, message: &str, highlight_color: &'static str) -> String {
        let mut highlight = false;
        message
            .chars()
            .map(|c| {
                let cs = c.to_string();
                ternary!(c != '`', cs, {
                    highlight = !highlight;
                    ternary!(
                        highlight,
                        cs + &highlight_color.bold(),
                        cs.reset().bright_white()
                    )
                })
            })
            .collect()
    }

    pub fn annotate_snippet(&self, diag: &Diag) -> String {
        let mut data = self.emphasize(
            diag.details
                .iter()
                .filter(|detail| detail.span.src_id.is_valid())
                .map(|detail| {
                    let ptr = match &detail.kind {
                        DiagDetailKind::Primary(_) => '^',
                        DiagDetailKind::Note | DiagDetailKind::Help => '-',
                    };

                    EmphasisTarget::new(
                        detail.message.clone(),
                        detail.span,
                        (self.detail_color(&detail.kind), ptr),
                    )
                })
                .collect(),
        );

        let max_lno = data
            .iter()
            .flat_map(|data| data.lines.iter().map(|line| line.line_no))
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

        let cwd = std::env::current_dir()
            .unwrap_or_else(|_| panic!("failed to retrieve the current directory"));

        let formatted = data
            .into_iter()
            .flat_map(|data| {
                let lines = data.lines.into_iter().map(move |mut line| {
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
                            prev.and_then(|prev_idx| {
                                (idx == prev_idx).then_some(ptr.display.1.to_string())
                            })
                            .unwrap_or_else(|| ptr.display.1.to_string().style(ptr.display.0))
                        })
                        .chain(std::iter::once(style::RESET.into()))
                        .collect::<String>();

                    let messages = data
                        .messages
                        .get(&line.line_no)
                        .map(|messages| {
                            messages
                                .iter()
                                .enumerate()
                                .map(|(i, message)| {
                                    let mut msg = " ".repeat(n);
                                    messages
                                        .iter()
                                        .enumerate()
                                        .skip(i)
                                        .for_each(|(j, message)| {
                                            let offset = message.offset as usize - 1;
                                            let idx = line
                                                .content
                                                .char_indices()
                                                .nth(offset)
                                                .map(|(idx, _)| idx)
                                                .unwrap_or(n);
                                            ternary!(
                                                i == j,
                                                msg.insert_str(
                                                    idx,
                                                    &message.content.style(message.color).bold(),
                                                ),
                                                msg.chars()
                                                    .nth(idx)
                                                    .is_some_and(char::is_whitespace)
                                                    .then(|| {
                                                        msg.replace_range(
                                                            idx..(idx + 1),
                                                            &"|".style(message.color).bold(),
                                                        )
                                                    })
                                                    .unwrap_or(())
                                            )
                                        });
                                    msg
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();

                    let pref = pref_fn(None);

                    format!(
                        "{lined_pref}{content}\n{pref}{ptr}{message}",
                        lined_pref = pref_fn(Some(line.line_no)),
                        content = line.content,
                        ptr = pointer.bold(),
                        message = ternary!(
                            messages.is_empty(),
                            "".into(),
                            messages
                                .iter()
                                .map(|message| format!("\n{pref}{message}"))
                                .collect::<Vec<_>>()
                                .join("")
                        )
                    )
                });

                let source = self.source_registry.get(data.span.src_id);
                let (start, _) = data.span.to_position_range(source);

                std::iter::once(format!(
                    " {arrow} {reset}{file}:{pos}{reset}",
                    arrow = "--->".bright_blue().bold(),
                    reset = style::RESET,
                    file =
                        source.identifier.1.replace(cwd.to_str().unwrap(), "")[1..].bright_black(),
                    pos = format!("{}:{}", start.line, start.column),
                ))
                .chain(lines)
            })
            .collect::<Vec<_>>();

        formatted.join("\n")
    }
}

impl<'r> DiagnosticReporter for CLIReporter<'r> {
    fn format_diagnostic(&self, diag: &Diag) -> String {
        let (s_kind, code) = diag.kind.data();
        let sev_color = self.color(&diag.kind);

        format!(
            "{sev}{code}{reset} {message}\n{snippet}\n{reset}{sub_diags}{reset}",
            sev = s_kind.style(&sev_color).bold(),
            code = code.map_or_else(|| ":".into(), |code| format!("<E{code:04}>")),
            message = self.highlight(&diag.message, sev_color),
            snippet = self.annotate_snippet(&diag),
            sub_diags = diag
                .sub_diagnostics
                .iter()
                .map(|sub| self.format_diagnostic(&sub))
                .collect::<String>(),
            reset = style::RESET
        )
    }

    fn report(&self) {
        println!(
            "{}",
            self.dctx
                .data()
                .iter()
                .map(|diag| self.format_diagnostic(&diag))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}
