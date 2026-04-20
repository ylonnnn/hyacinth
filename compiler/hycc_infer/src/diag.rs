use hycc_diagnostic::{
    Diagnostic, DiagnosticContext, DiagnosticCtx,
    diagnostic::{Diag, DiagNoteKind, DiagnosticKind},
};
use hycc_span::Span;

#[derive(Debug)]
pub struct InfererDiagDataCtx {
    // pub interner: &'i SymbolInterner,
    // pub definitions: &'d DefinitionTable,
    // pub hir_table: &'t HirTable<'h>,
    // pub scope_ctx: &'s ScopeCtx,
}

impl InfererDiagDataCtx {
    pub fn new(// interner: &'i SymbolInterner,
        // definitions: &'d DefinitionTable,
        // hir_table: &'t HirTable<'h>,
        // scope_ctx: &'s ScopeCtx,
    ) -> Self {
        Self {
            // interner,
            // definitions,
            // hir_table,
            // scope_ctx,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InfererDiagCtx(Vec<InfererDiag>, bool);

impl InfererDiagCtx {
    #[allow(unused)]
    const RESOLVER_NOTE_OFFSET: u16 = 200;
    #[allow(unused)]
    const RESOLVER_WARNING_OFFSET: u16 = 300;
    const RESOLVER_ERROR_OFFSET: u16 = 450;

    pub fn new() -> Self {
        Self(Vec::new(), false)
    }

    pub fn warning(&mut self, span: Span, kind: InfererDiagWarningKind) {
        self.add(InfererDiag {
            span,
            kind: InfererDiagKind::Warning(kind),
        });
    }

    pub fn error(&mut self, span: Span, kind: InfererDiagErrorKind) {
        self.add(InfererDiag {
            span,
            kind: InfererDiagKind::Error(kind),
        });
    }
}

impl DiagnosticContext<InfererDiagDataCtx, InfererDiag> for InfererDiagCtx {
    fn data(&self) -> &Vec<InfererDiag> {
        &self.0
    }

    fn data_mut(&mut self) -> &mut Vec<InfererDiag> {
        &mut self.0
    }

    fn error_occurred(&self) -> bool {
        self.1
    }

    fn emit(&self, target: &mut DiagnosticCtx, ctx: InfererDiagDataCtx) {
        for diag in self.data() {
            target.add(diag.emit(&ctx));
        }
    }
}

#[derive(Debug, Clone)]
pub enum InfererDiagKind {
    Note(DiagNoteKind),
    Warning(InfererDiagWarningKind),
    Error(InfererDiagErrorKind),
}

#[derive(Debug, Clone)]
pub enum InfererDiagWarningKind {}

#[derive(Debug, Clone)]
pub enum InfererDiagErrorKind {}

#[derive(Debug, Clone)]
pub struct InfererDiag {
    pub kind: InfererDiagKind,
    pub span: Span,
}

impl InfererDiag {
    pub fn warning(span: Span, kind: InfererDiagWarningKind) -> Self {
        Self {
            span,
            kind: InfererDiagKind::Warning(kind),
        }
    }

    pub fn error(span: Span, kind: InfererDiagErrorKind) -> Self {
        Self {
            span,
            kind: InfererDiagKind::Error(kind),
        }
    }
}

impl Diag<InfererDiagDataCtx> for InfererDiag {
    fn emit(&self, ctx: &InfererDiagDataCtx) -> Diagnostic {
        use InfererDiagErrorKind as Err;
        use InfererDiagKind::*;

        let InfererDiagDataCtx { .. } = *ctx;

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
                    let code = (unsafe { *(&self.kind as *const InfererDiagKind as *const u8) })
                        as u16
                        + InfererDiagCtx::RESOLVER_ERROR_OFFSET;

                    DiagnosticKind::Error(
                        code,
                        match kind {
                            _ => "".into(),
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
                #[allow(unreachable_patterns)]
                _ => {}
            },
        }

        diag
    }
}
