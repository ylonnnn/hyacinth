use hycc_diagnostic::{
    Diagnostic, DiagnosticContext, DiagnosticCtx,
    diagnostic::{Diag, DiagNoteKind, DiagnosticKind},
};
use hycc_hir::def::{DefId, DefSpace, DefinitionTable};
// use hycc_hir::{
//     HirTable,
//     def::{DefId, DefinitionTable},
// };
// use hycc_scope::ScopeCtx;
use hycc_span::Span;
use hycc_symbol::{Symbol, SymbolInterner};
// use hycc_symbol::{Symbol, SymbolInterner};

#[derive(Debug)]
pub struct ResolverDiagDataCtx<'i, 'd> {
    pub interner: &'i SymbolInterner,
    pub definitions: &'d DefinitionTable,
    // pub hir_table: &'t HirTable<'h>,
    // pub scope_ctx: &'s ScopeCtx,
}

impl<'i, 'd> ResolverDiagDataCtx<'i, 'd> {
    pub fn new(
        interner: &'i SymbolInterner,
        definitions: &'d DefinitionTable,
        // hir_table: &'t HirTable<'h>,
        // scope_ctx: &'s ScopeCtx,
    ) -> Self {
        Self {
            interner,
            definitions,
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

impl<'i, 'd> DiagnosticContext<ResolverDiagDataCtx<'i, 'd>, ResolverDiag> for ResolverDiagCtx {
    fn data(&self) -> &Vec<ResolverDiag> {
        &self.0
    }

    fn data_mut(&mut self) -> &mut Vec<ResolverDiag> {
        &mut self.0
    }

    fn error_occurred(&self) -> bool {
        self.1
    }

    fn emit(&self, target: &mut DiagnosticCtx, ctx: ResolverDiagDataCtx) {
        for diag in self.data() {
            target.add(diag.emit(&ctx));
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
    InvalidPetalResolution(Symbol, DefId),
    InvalidInference,
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

impl<'i, 'd> Diag<ResolverDiagDataCtx<'i, 'd>> for ResolverDiag {
    fn emit(&self, ctx: &ResolverDiagDataCtx) -> Diagnostic {
        use ResolverDiagErrorKind as Err;
        use ResolverDiagKind::*;

        let ResolverDiagDataCtx {
            interner,
            definitions,
            ..
        } = *ctx;

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
                                    interner.get(*symbol)
                                )
                            }

                            Err::InvalidPetalResolution(name, _) => {
                                format!("cannot resolve petal `{}` as type.", interner.get(*name))
                            }

                            Err::InvalidInference => {
                                format!("cannot infer type in this context.")
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
                    let def = definitions.get(*def_id);

                    diag.detail(
                        def.span,
                        DiagnosticKind::Note(format!(
                            "`{}` is defined as a petal here.",
                            interner.get(*name)
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
