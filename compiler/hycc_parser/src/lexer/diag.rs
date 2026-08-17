use hycc_diagnostic::diagnostic::{Diag, DiagCtx, DiagEmitter, DiagKind, DiagLike, Diagnostics};
use hycc_source::SourceRegistry;
use hycc_span::Span;

#[derive(Debug, Clone)]
pub struct LexerDiagDataCtx<'c> {
    pub registry: &'c SourceRegistry,
}

impl<'c> LexerDiagDataCtx<'c> {
    pub fn new(registry: &'c SourceRegistry) -> Self {
        Self { registry }
    }
}

#[derive(Debug)]
pub struct LexerDiagCtx<'c>(Vec<LexerDiag>, &'c mut DiagCtx, bool);

impl<'c> LexerDiagCtx<'c> {
    pub fn new(dctx: &'c mut DiagCtx) -> Self {
        Self(Vec::new(), dctx, false)
    }

    pub fn error(&mut self, span: Span, kind: LexerDiagErrorKind) {
        self.add(LexerDiag {
            span,
            kind: LexerDiagKind::Error(kind),
        })
    }
}

impl<'c> Diagnostics<LexerDiagDataCtx<'c>, LexerDiag> for LexerDiagCtx<'c> {
    const ERROR_CODE_OFFSET: u16 = 200;

    fn data(&self) -> &[LexerDiag] {
        &self.0
    }

    fn data_mut(&mut self) -> &mut Vec<LexerDiag> {
        &mut self.0
    }

    fn error_flag(&mut self) -> &mut bool {
        &mut self.2
    }

    fn emit(&mut self, mut ctx: LexerDiagDataCtx) {
        for diag in &self.0 {
            self.1.add(diag.emit(&mut ctx));
        }
    }
}

#[repr(u16)]
#[derive(Debug, Clone)]
pub enum LexerDiagErrorKind {
    InvalidNumericLiteralDigit { digit: u8, base: u8 },
    InvalidNumericLiteralPrefix,
    DanglingNumericLiteralPrefix,

    UnterminatedCharSeq,
    InvalidCharSeq { enclosing: u8, len: (usize, usize) },

    UnclosedDelimeterCollection { op: u8, cl: u8 },

    UnknownChar { c: u8 },
}

#[derive(Debug, Clone)]
pub enum LexerDiagKind {
    Info,
    Warning,
    Error(LexerDiagErrorKind),
}

#[derive(Debug)]
pub struct LexerDiag {
    kind: LexerDiagKind,
    span: Span,
}

impl DiagLike for LexerDiag {
    fn is_info(&self) -> bool {
        matches!(&self.kind, LexerDiagKind::Info)
    }

    fn is_warning(&self) -> bool {
        matches!(&self.kind, LexerDiagKind::Warning)
    }

    fn is_error(&self) -> bool {
        matches!(&self.kind, LexerDiagKind::Error(_))
    }
}

impl<'c> DiagEmitter<LexerDiagDataCtx<'c>> for LexerDiag {
    fn emit(&self, ctx: &mut LexerDiagDataCtx) -> Diag {
        use LexerDiagErrorKind::*;

        let source = ctx.registry.get(self.span.src_id);
        let (kind, message) = match &self.kind {
            LexerDiagKind::Info => (DiagKind::Info, "".into()),
            LexerDiagKind::Warning => (DiagKind::Warning, "".into()),

            LexerDiagKind::Error(kind) => {
                let message = match &kind {
                    InvalidNumericLiteralDigit { digit, base } => format!(
                        "invalid numeric digit `{}` for numeric literals with base `{}`",
                        *digit as char, *base
                    ),

                    InvalidNumericLiteralPrefix => format!(
                        "invalid numeric literal prefix `{}`.",
                        &source.data[(self.span.offset as usize)
                            ..=((self.span.offset + self.span.len as u32) as usize)]
                    ),

                    DanglingNumericLiteralPrefix => {
                        format!(
                            "dangling umeric literal prefix `{}`.",
                            &source.data[(self.span.offset as usize)
                                ..=((self.span.offset + self.span.len as u32) as usize)]
                        )
                    }

                    UnterminatedCharSeq => {
                        format!("unterminated character sequence.")
                    }

                    InvalidCharSeq { enclosing, len } => format!(
                        "character sequences within `{}` expect `{}` characters, received `{}`.",
                        *enclosing as char, len.0, len.1
                    ),

                    UnclosedDelimeterCollection { op, cl } => {
                        format!("missing closing `{}` for `{}`.", *cl as char, *op as char)
                    }

                    UnknownChar { c } => format!("unknown character `{}`.", *c as char),
                };

                (
                    DiagKind::Error(
                        hycc_util::enums::tag_of::<u16, _>(&kind) + LexerDiagCtx::ERROR_CODE_OFFSET,
                    ),
                    message,
                )
            }

            _ => todo!("emit non-error lexer diagnostics"),
        };

        Diag::new(kind, self.span, message)
    }
}
