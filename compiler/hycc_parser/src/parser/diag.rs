use std::path::{self, PathBuf};

use hycc_ast::{
    item::ItemKind,
    token::{Token, TokenKind},
};
use hycc_diagnostic::diagnostic::{Diag, DiagCtx, DiagEmitter, DiagKind, DiagLike, Diagnostics};
use hycc_session::config;
use hycc_source::SourceRegistry;
use hycc_span::Span;
use hycc_util::ternary;

pub type ParseResult<T, E = Option<ParserDiag>> = Result<T, E>;

#[derive(Debug, Clone)]
pub struct ParserDiagDataCtx<'s> {
    pub registry: &'s SourceRegistry,
}

impl<'s> ParserDiagDataCtx<'s> {
    pub fn new(registry: &'s SourceRegistry) -> Self {
        Self { registry }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserDiagCtxState {
    Synchronized,
    Disarray,
}

#[derive(Debug)]
pub struct ParserDiagCtx<'c> {
    data: Vec<ParserDiag>,
    dctx: &'c mut DiagCtx,
    state: ParserDiagCtxState,
    errored: bool,
}

impl<'c> ParserDiagCtx<'c> {
    pub fn new(dctx: &'c mut DiagCtx) -> Self {
        Self {
            data: Vec::new(),
            dctx,
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

    pub fn error(&mut self, span: Span, kind: ParserDiagErrorKind) {
        self.add(ParserDiag {
            span,
            kind: ParserDiagKind::Error(kind),
        });
    }
}

impl<'c> Diagnostics<ParserDiagDataCtx<'c>, ParserDiag> for ParserDiagCtx<'c> {
    const ERROR_CODE_OFFSET: u16 = 300;

    fn data(&self) -> &[ParserDiag] {
        &self.data
    }

    fn data_mut(&mut self) -> &mut Vec<ParserDiag> {
        &mut self.data
    }

    fn error_flag(&mut self) -> &mut bool {
        &mut self.errored
    }

    fn add(&mut self, diag: ParserDiag) {
        let is_err = diag.is_error();
        if is_err {
            if self.is(ParserDiagCtxState::Disarray) {
                return;
            }

            self.state = ParserDiagCtxState::Disarray;
            self.errored = true;
        }

        self.data_mut().push(diag);
    }

    fn emit(&mut self, mut ctx: ParserDiagDataCtx<'c>) {
        for diag in &self.data {
            self.dctx.add(diag.emit(&mut ctx));
        }
    }
}

#[derive(Debug, Clone)]
pub enum ParserDiagKind {
    Info,
    Warning,
    Error(ParserDiagErrorKind),
}

#[repr(u16)]
#[derive(Debug, Clone)]
pub enum ParserDiagErrorKind {
    UnexpectedToken {
        token: Token,
        expected: Option<UnexpectedTokenExpectation>,
    } = 0,

    InvalidVarDecl {
        ident: Token,
        depth: usize,
    },

    UnrecognizedPetalFile {
        path: PathBuf,
    },

    IllegalLocalNonInlinePetalDeclaration,

    InvalidStructFieldCount(u8),
    UnsupportedItem {
        item_kind: ItemKind,
        context: &'static str,
    }, // TODO: update `context` to
       // something more optimal
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

#[derive(Debug, Clone)]
pub struct ParserDiag {
    pub kind: ParserDiagKind,
    pub span: Span,
}

impl<'s> ParserDiag {
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

impl DiagLike for ParserDiag {
    fn is_info(&self) -> bool {
        matches!(&self.kind, ParserDiagKind::Info)
    }

    fn is_warning(&self) -> bool {
        matches!(&self.kind, ParserDiagKind::Warning)
    }

    fn is_error(&self) -> bool {
        matches!(&self.kind, ParserDiagKind::Error(_))
    }
}

impl<'c> DiagEmitter<ParserDiagDataCtx<'c>> for ParserDiag {
    fn emit(&self, ctx: &mut ParserDiagDataCtx<'c>) -> Diag {
        use ParserDiagErrorKind::*;

        let source = ctx.registry.get(self.span.src_id);
        // let code = (unsafe { *(&self.kind as *const ParserDiagKind as *const u8) }) as u16
        //     + ParserDiagCtx::PARSER_ERROR_OFFSET;

        let (kind, message) = match &self.kind {
            ParserDiagKind::Info => (DiagKind::Info, "".into()),
            ParserDiagKind::Warning => (DiagKind::Warning, "".into()),

            ParserDiagKind::Error(kind) => {
                let message = match kind {
                    UnexpectedToken { token, expected } => {
                        format!(
                            "unexpected `{}`{}.",
                            token.view(&source.data),
                            ternary!(
                                expected.is_some(),
                                format!(", expected `{}`", expected.as_ref().unwrap().to_string()),
                                "".into()
                            )
                        )
                    }

                    InvalidVarDecl { ident, depth } => {
                        let message = format!(
                            "invalid {}variable declaration for `{}`.",
                            ternary!(*depth == 0, "top-level ", ""),
                            ident.view(&source.data)
                        );

                        message
                    }

                    UnrecognizedPetalFile { path } => {
                        format!(
                            "cannot find corresponding petal file for petal `{}`.",
                            path.to_str().unwrap().replace(
                                path::MAIN_SEPARATOR_STR,
                                &config::HYC_PATH_SEP_TOK_KIND.to_string()
                            )
                        )
                    }

                    IllegalLocalNonInlinePetalDeclaration => {
                        format!("cannot declare non-inline petals locally within local blocks.")
                    }

                    InvalidStructFieldCount(n) => {
                        format!(
                            "structs cannot have more than `{}` fields, found `{}`.",
                            config::HYC_STRUCT_FIELD_LIMIT,
                            n
                        )
                    }

                    UnsupportedItem { item_kind, context } => format!(
                        "`{}`s are unsupported within `{context}`s.",
                        item_kind.kind()
                    ),
                };

                (
                    DiagKind::Error(
                        hycc_util::enums::tag_of::<u16, _>(&kind)
                            + ParserDiagCtx::ERROR_CODE_OFFSET,
                    ),
                    message,
                )
            }
        };

        let mut diag = Diag::new(kind, self.span, message);

        match &self.kind {
            ParserDiagKind::Error(kind) => match kind {
                InvalidVarDecl { depth, .. } => {
                    // diag.detail(diag.span, DiagKind::Note(
                    //         ternary!(*depth == 0,
                    //             format!("top-level variable declarations must have both `explicit type annotation` and a `constant initializer value`."),
                    //             format!("variable declarations must either have an `explicit type annotation` or an `initializer value`.")
                    // )));
                }

                UnrecognizedPetalFile { path } => {
                    let petal = path.to_str().unwrap();

                    // diag.detail(
                    //     diag.span,
                    //     DiagKind::Note(format!(
                    //         "create petal file `{}` or `{}` relative to the current file.",
                    //         format_args!("{}.{}", petal, config::HYC_FILE_EXT),
                    //         format_args!("{}/{}", petal, config::HYC_DIR_PETAL_FILE)
                    //     )),
                    // );
                }

                _ => {}
            },

            _ => {}
        }

        diag
    }
}
