use hycc_diagnostic::{
    Diagnostic, DiagnosticContext, DiagnosticCtx,
    diagnostic::{Diag, DiagNoteKind, DiagnosticKind},
};
// use hycc_hir::{
//     HirTable,
//     def::{DefId, DefinitionTable},
// };
// use hycc_scope::ScopeCtx;
use hycc_span::Span;
// use hycc_symbol::{Symbol, SymbolInterner};

#[derive(Debug)]
pub struct ResolverDiagDataCtx {
    // pub interner: &'i SymbolInterner,
    // pub hir_table: &'t HirTable<'h>,
    // pub definitions: &'d DefinitionTable,
    // pub scope_ctx: &'s ScopeCtx,
}

impl ResolverDiagDataCtx {
    pub fn new(// interner: &'i SymbolInterner,
        // hir_table: &'t HirTable<'h>,
        // definitions: &'d DefinitionTable,
        // scope_ctx: &'s ScopeCtx,
    ) -> Self {
        Self {
            // interner,
            // hir_table,
            // definitions,
            // scope_ctx,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolverDiagCtx(Vec<ResolverDiag>, bool);

impl ResolverDiagCtx {
    #[allow(unused)]
    const COLLECTOR_NOTE_OFFSET: u16 = 200;
    #[allow(unused)]
    const COLLECTOR_WARNING_OFFSET: u16 = 300;
    const COLLECTOR_ERROR_OFFSET: u16 = 450;

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

impl DiagnosticContext<ResolverDiagDataCtx, ResolverDiag> for ResolverDiagCtx {
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
pub enum ResolverDiagErrorKind {}

#[derive(Debug, Clone)]
pub struct ResolverDiag {
    pub kind: ResolverDiagKind,
    pub span: Span,
}

impl ResolverDiag {
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

impl Diag<ResolverDiagDataCtx> for ResolverDiag {
    fn emit(&self, ctx: &ResolverDiagDataCtx) -> Diagnostic {
        // use ResolverDiagErrorKind as Err;
        use ResolverDiagKind::*;

        let ResolverDiagDataCtx { .. } = *ctx;
        let code = (unsafe { *(&self.kind as *const ResolverDiagKind as *const u8) }) as u16
            + ResolverDiagCtx::COLLECTOR_ERROR_OFFSET;

        let diag = Diagnostic::new(
            self.span,
            match &self.kind {
                Note(kind) => DiagnosticKind::Note(match kind {
                    _ => "".into(),
                }),

                Warning(kind) => DiagnosticKind::Warning(match kind {
                    _ => "".into(),
                }),

                Error(kind) => DiagnosticKind::Error(
                    code,
                    match kind {
                        _ => "".into(),
                    },
                ),
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
                #[allow(unreachable_patterns)]
                _ => {}
            },
        }

        diag
    }
}
