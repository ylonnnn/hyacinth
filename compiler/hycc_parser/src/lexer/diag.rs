use hycc_diagnostic::{
    Diagnostic, DiagnosticContext, DiagnosticCtx,
    diagnostic::{Diag, DiagnosticKind},
};
use hycc_source::SourceRegistry;
use hycc_span::Span;

#[derive(Debug, Clone)]
pub struct LexerDiagDataCtx<'s> {
    pub registry: &'s SourceRegistry,
}

impl<'s> LexerDiagDataCtx<'s> {
    pub fn new(registry: &'s SourceRegistry) -> Self {
        Self { registry }
    }
}

#[derive(Debug, Clone)]
pub struct LexerDiagCtx(Vec<LexerDiag>, bool);

impl LexerDiagCtx {
    #[allow(unused)]
    const LEXER_NOTE_OFFSET: u16 = 200;
    #[allow(unused)]
    const LEXER_WARNING_OFFSET: u16 = 300;
    const LEXER_ERROR_OFFSET: u16 = 400;

    pub fn new() -> Self {
        Self(Vec::new(), false)
    }

    pub fn warning(&mut self, span: Span, kind: LexerDiagWarningKind) {
        self.add(LexerDiag {
            span,
            kind: LexerDiagKind::Warning(kind),
        });
    }

    pub fn error(&mut self, span: Span, kind: LexerDiagErrorKind) {
        self.add(LexerDiag {
            span,
            kind: LexerDiagKind::Error(kind),
        });
    }
}

impl<'s> DiagnosticContext<LexerDiagDataCtx<'s>, LexerDiag> for LexerDiagCtx {
    fn data(&self) -> &Vec<LexerDiag> {
        &self.0
    }

    fn data_mut(&mut self) -> &mut Vec<LexerDiag> {
        &mut self.0
    }

    fn error_occurred(&self) -> bool {
        self.1
    }

    fn emit(&self, target: &mut DiagnosticCtx, ctx: LexerDiagDataCtx<'s>) {
        for diag in self.data() {
            target.add(diag.emit(&ctx));
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum LexerDiagKind {
    Warning(LexerDiagWarningKind),
    Error(LexerDiagErrorKind),
}

#[derive(Debug, Clone)]
pub enum LexerDiagWarningKind {}

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum LexerDiagErrorKind {
    InvalidNumericLiteralDigit { digit: u8, base: u8 },
    InvalidNumericLiteralPrefix,
    DanglingNumericLiteralPrefix,
    InvalidLiteral,
    UnterminatedCharSeq,
    InvalidCharSeq { enclosing: u8, len: (usize, usize) },
    UnclosedDelimeterCollection { op: u8, cl: u8 },
    UnknownChar { c: u8 },
}

#[derive(Debug, Clone)]
pub struct LexerDiag {
    pub kind: LexerDiagKind,
    pub span: Span,
}

impl<'s> Diag<LexerDiagDataCtx<'s>> for LexerDiag {
    fn emit(&self, ctx: &LexerDiagDataCtx<'s>) -> Diagnostic {
        use LexerDiagErrorKind as Err;
        use LexerDiagKind::*;

        let LexerDiagDataCtx::<'s> { registry } = *ctx;
        let source = registry.get(self.span.src_id);
        let code = (unsafe { *(&self.kind as *const LexerDiagKind as *const u8) }) as u16
            + LexerDiagCtx::LEXER_ERROR_OFFSET;

        let kind = match &self.kind {
            Warning(kind) => DiagnosticKind::Warning(
                code,
                match kind {
                    _ => "".into(),
                },
            ),

            Error(kind) => DiagnosticKind::Error(
                code,
                match kind {
                    Err::InvalidNumericLiteralDigit { digit, base } => format!(
                        "invalid numeric digit `{}` for numeric literals with base `{}`",
                        *digit as char, *base
                    ),

                    Err::InvalidNumericLiteralPrefix => format!(
                        "invalid numeric literal prefix `{}`.",
                        &source.data[(self.span.offset as usize)
                            ..=((self.span.offset + self.span.len as u32) as usize)]
                    ),

                    Err::DanglingNumericLiteralPrefix => {
                        format!(
                            "dangling umeric literal prefix `{}`.",
                            &source.data[(self.span.offset as usize)
                                ..=((self.span.offset + self.span.len as u32) as usize)]
                        )
                    }

                    Err::UnterminatedCharSeq => {
                        format!("unterminated character sequence.")
                    }

                    Err::InvalidCharSeq { enclosing, len } => format!(
                        "character sequences within `{}` expect `{}` characters, received `{}`.",
                        *enclosing as char, len.0, len.1
                    ),

                    Err::UnclosedDelimeterCollection { op, cl } => {
                        format!("missing closing `{}` for `{}`.", *cl as char, *op as char)
                    }

                    Err::UnknownChar { c } => format!("unknown character `{}`.", *c as char),

                    _ => todo!(),
                },
            ),
        };

        Diagnostic::new(self.span, kind)
    }
}
