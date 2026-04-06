use hycc_ast::token::{Token, TokenKind};
use hycc_diagnostic::{
    Diagnostic, DiagnosticContext, DiagnosticCtx,
    diagnostic::{Diag, DiagnosticKind},
};
use hycc_source::SourceRegistry;
use hycc_span::Span;
use hycc_util::ternary;

#[derive(Debug)]
pub struct ParserDiagCtx {
    data: Vec<ParserDiag>,
    state: ParserDiagCtxState,
    errored: bool,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserDiagCtxState {
    Synchronized,
    Disarray,
}

impl ParserDiagCtx {
    #[allow(unused)]
    const PARSER_NOTE_OFFSET: u16 = 200;
    #[allow(unused)]
    const PARSER_WARNING_OFFSET: u16 = 300;
    const PARSER_ERROR_OFFSET: u16 = 420;

    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            state: ParserDiagCtxState::Synchronized,
            errored: false,
        }
    }

    pub fn sync(&mut self) {
        self.state = ParserDiagCtxState::Synchronized
    }

    pub fn is(&self, state: ParserDiagCtxState) -> bool {
        self.state == state
    }

    pub fn is_in_disarray(&self) -> bool {
        self.is(ParserDiagCtxState::Disarray)
    }

    pub fn warning(&mut self, span: Span, kind: ParserDiagWarningKind) {
        self.add(ParserDiag {
            span,
            kind: ParserDiagKind::Warning(kind),
        });
    }

    pub fn error(&mut self, span: Span, kind: ParserDiagErrorKind) {
        self.add(ParserDiag {
            span,
            kind: ParserDiagKind::Error(kind),
        });
    }
}

impl DiagnosticContext<&SourceRegistry, ParserDiag> for ParserDiagCtx {
    fn data(&self) -> &Vec<ParserDiag> {
        &self.data
    }

    fn data_mut(&mut self) -> &mut Vec<ParserDiag> {
        &mut self.data
    }

    fn error_occurred(&self) -> bool {
        self.errored
    }

    fn add(&mut self, diagnostic: ParserDiag) -> Option<&mut ParserDiag> {
        let is_err = diagnostic.kind.is_error();
        if is_err {
            if self.is(ParserDiagCtxState::Disarray) {
                return None;
            }

            self.state = ParserDiagCtxState::Disarray;
            self.errored = true;
        }

        let data = self.data_mut();

        data.push(diagnostic);
        data.last_mut()
    }

    fn emit(&self, target: &mut DiagnosticCtx, ctx: &SourceRegistry) {
        for diag in self.data() {
            target.add(diag.emit(ctx));
        }
    }
}

#[derive(Debug, Clone)]
pub enum ParserDiagKind {
    Warning(ParserDiagWarningKind),
    Error(ParserDiagErrorKind),
}

#[derive(Debug, Clone)]
pub enum ParserDiagWarningKind {}

#[derive(Debug, Clone)]
pub enum ParserDiagErrorKind {
    UnexpectedToken {
        token: Token,
        expected: Option<UnexpectedTokenExpectation>,
    },

    InvalidVarDecl {
        ident: Token,
    },
}

#[derive(Debug, Clone)]
pub enum UnexpectedTokenExpectation {
    Arbitrary(&'static str),
    TokenKind(TokenKind),
}

impl ToString for UnexpectedTokenExpectation {
    fn to_string(&self) -> String {
        match self {
            Self::Arbitrary(s) => (*s).into(),
            Self::TokenKind(kind) => kind.to_string(),
        }
    }
}

impl ParserDiagKind {
    pub fn is_warning(&self) -> bool {
        matches!(self, Self::Warning(..))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(..))
    }
}

#[derive(Debug, Clone)]
pub struct ParserDiag {
    pub kind: ParserDiagKind,
    pub span: Span,
}

impl ParserDiag {
    pub fn warning(span: Span, kind: ParserDiagWarningKind) -> Self {
        Self {
            span,
            kind: ParserDiagKind::Warning(kind),
        }
    }

    pub fn error(span: Span, kind: ParserDiagErrorKind) -> Self {
        Self {
            span,
            kind: ParserDiagKind::Error(kind),
        }
    }

    pub fn unexpected_token(token: Token) -> Self {
        Self::error(
            token.span,
            ParserDiagErrorKind::UnexpectedToken {
                token,
                expected: None,
            },
        )
    }

    pub fn unexpected_token_expected_arbitrary(token: Token, expected: &'static str) -> Self {
        Self::error(
            token.span,
            ParserDiagErrorKind::UnexpectedToken {
                token,
                expected: Some(UnexpectedTokenExpectation::Arbitrary(expected)),
            },
        )
    }

    pub fn unexpected_token_expected_token(token: Token, expected: TokenKind) -> Self {
        Self::error(
            token.span,
            ParserDiagErrorKind::UnexpectedToken {
                token,
                expected: Some(UnexpectedTokenExpectation::TokenKind(expected)),
            },
        )
    }
}

impl Diag<&SourceRegistry> for ParserDiag {
    fn emit(&self, ctx: &SourceRegistry) -> Diagnostic {
        use ParserDiagErrorKind as Err;
        use ParserDiagKind::*;

        let registry = ctx;
        let source = registry.get(self.span.src_id);
        let code = (unsafe { *(&self.kind as *const ParserDiagKind as *const u8) }) as u16
            + ParserDiagCtx::PARSER_ERROR_OFFSET;

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
                    Err::UnexpectedToken { token, expected } => {
                        format!(
                            "unexpected `{}`{}.",
                            token.view(&source.data),
                            ternary!(
                                expected.is_some(),
                                format!(", expected {}", expected.as_ref().unwrap().to_string()),
                                "".into()
                            )
                        )
                    }

                    Err::InvalidVarDecl { ident } => {
                        let message = format!(
                            "invalid variable declaration for `{}`.",
                            ident.view(&source.data)
                        );
                        // TODO: additional detail/note

                        message
                    }
                },
            ),
        };

        Diagnostic::new(self.span, kind)
    }
}
