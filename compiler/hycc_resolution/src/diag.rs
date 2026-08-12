use hycc_diagnostic::{
    Diagnostic, DiagnosticContext, DiagnosticCtx,
    diagnostic::{Diag, DiagNoteKind, DiagnosticKind},
};
use hycc_hir::def::{DefId, DefSpace, DefinitionTable};
use hycc_span::Span;
use hycc_symbol::{Symbol, SymbolInterner};
use hycc_ty::{
    context::{TyCtx, TyId},
    fmt::TyFormatter,
};
use hycc_util::ternary;

#[derive(Debug)]
pub struct ResolverDiagDataCtx<'t, 'd, 'i> {
    pub fmt: TyFormatter<'t, 'd, 'i>,
    // pub hir_table: &'t HirTable<'h>,
    // pub scope_ctx: &'s ScopeCtx,
}

impl<'t, 'd, 'i> ResolverDiagDataCtx<'t, 'd, 'i> {
    pub fn new(
        tctx: &'t mut TyCtx,
        definitions: &'d DefinitionTable,
        interner: &'i SymbolInterner,
        // hir_table: &'t HirTable<'h>,
        // scope_ctx: &'s ScopeCtx,
    ) -> Self {
        Self {
            fmt: TyFormatter::new(tctx, &definitions, &interner),
            // hir_table,
            // scope_ctx,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolverDiagCtx(Vec<ResolverDiag>, bool);

impl<'c> ResolverDiagCtx {
    #[allow(unused)]
    const RESOLVER_NOTE_OFFSET: u16 = 200;
    #[allow(unused)]
    const RESOLVER_WARNING_OFFSET: u16 = 300;
    const RESOLVER_ERROR_OFFSET: u16 = 450;

    pub fn new() -> Self {
        Self(Vec::new(), false)
    }

    pub fn warning(&mut self, span: Span, kind: ResolverDiagWarningKind) {
        self.add(ResolverDiag {
            span,
            kind: ResolverDiagKind::Warning(kind),
        });
    }

    pub fn error(&mut self, span: Span, kind: ResolverDiagErrorKind) {
        self.add(ResolverDiag {
            span,
            kind: ResolverDiagKind::Error(kind),
        });
    }
}

impl<'t, 'd, 'i> DiagnosticContext<ResolverDiagDataCtx<'t, 'd, 'i>, ResolverDiag>
    for ResolverDiagCtx
{
    fn data(&self) -> &Vec<ResolverDiag> {
        &self.0
    }

    fn data_mut(&mut self) -> &mut Vec<ResolverDiag> {
        &mut self.0
    }

    fn error_occurred(&self) -> bool {
        self.1
    }

    fn emit(&self, target: &mut DiagnosticCtx, mut ctx: ResolverDiagDataCtx) {
        for diag in self.data() {
            target.add(diag.emit(&mut ctx));
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResolverDiagKind {
    Note(DiagNoteKind),
    Warning(ResolverDiagWarningKind),
    Error(ResolverDiagErrorKind),
}

#[derive(Debug, Clone)]
pub enum ResolverDiagWarningKind {}

#[derive(Debug, Clone)]
pub enum ResolverDiagErrorKind {
    UnrecognizedSymbol(Symbol, Option<DefSpace>),
    UnrecognizedMember { name: Symbol, ty_id: TyId },

    InvalidPetalResolution(Symbol, DefId),
    InvalidInference,
    InaccessibleSymbol(Symbol),

    GenericArgumentArityMismatch(u16),
}

#[derive(Debug, Clone)]
pub struct ResolverDiag {
    pub kind: ResolverDiagKind,
    pub span: Span,
}

impl<'c> ResolverDiag {
    pub fn warning(span: Span, kind: ResolverDiagWarningKind) -> Self {
        Self {
            span,
            kind: ResolverDiagKind::Warning(kind),
        }
    }

    pub fn error(span: Span, kind: ResolverDiagErrorKind) -> Self {
        Self {
            span,
            kind: ResolverDiagKind::Error(kind),
        }
    }
}

impl<'t, 'd, 'i> Diag<ResolverDiagDataCtx<'t, 'd, 'i>> for ResolverDiag {
    fn emit(&self, ctx: &mut ResolverDiagDataCtx) -> Diagnostic {
        use ResolverDiagErrorKind as Err;
        use ResolverDiagKind::*;

        let ResolverDiagDataCtx { fmt, .. } = ctx;
        let mut diag = Diagnostic::new(
            self.span,
            match &self.kind {
                Note(kind) => DiagnosticKind::Note(match kind {
                    _ => "".into(),
                }),

                Warning(kind) => DiagnosticKind::Warning(match kind {
                    _ => "".into(),
                }),

                Error(kind) => {
                    let code = (unsafe { *(&self.kind as *const ResolverDiagKind as *const u8) })
                        as u16
                        + ResolverDiagCtx::RESOLVER_ERROR_OFFSET;

                    DiagnosticKind::Error(
                        code,
                        match kind {
                            Err::UnrecognizedSymbol(symbol, space) => {
                                let space = space
                                    .map(|space| space.to_string())
                                    .unwrap_or(String::from("symbol"));
                                format!(
                                    "cannot resolve unrecognized {space} `{}`.",
                                    fmt.interner.get(*symbol)
                                )
                            }

                            Err::UnrecognizedMember { name, ty_id } => {
                                format!(
                                    "cannot recognize associated item `{}` from type `{}`.",
                                    fmt.interner.get(*name),
                                    fmt.fmt_id(*ty_id)
                                )
                            }

                            Err::InvalidPetalResolution(name, _) => {
                                format!(
                                    "cannot resolve petal `{}` as type.",
                                    fmt.interner.get(*name)
                                )
                            }

                            Err::InvalidInference => {
                                format!("cannot infer type in this context.")
                            }

                            Err::InaccessibleSymbol(symbol) => {
                                format!(
                                    "symbol `{}` is inaccessible in this context.",
                                    fmt.interner.get(*symbol)
                                )
                            }

                            Err::GenericArgumentArityMismatch(data) => {
                                let (expected, received) =
                                    ((*data >> u8::BITS) as u8, (*data & u8::MAX as u16) as u8);
                                format!(
                                    "expected at most `{}` generic argument{}, received `{}` generic argument{}.",
                                    expected,
                                    ternary!(expected == 1, "", "s"),
                                    received,
                                    ternary!(received == 1, "", "s"),
                                )
                            }
                        },
                    )
                }
            },
        );

        // Optionally add details
        match &self.kind {
            Note(kind) => match kind {
                _ => {}
            },

            Warning(kind) => match kind {
                _ => {}
            },

            Error(kind) => match kind {
                Err::InvalidPetalResolution(name, def_id) => {
                    let def = fmt.definitions.get(*def_id);

                    diag.detail(
                        def.span,
                        DiagnosticKind::Note(format!(
                            "`{}` is defined as a petal here.",
                            fmt.interner.get(*name)
                        )),
                    );
                }

                #[allow(unreachable_patterns)]
                _ => {}
            },
        }

        diag
    }
}
