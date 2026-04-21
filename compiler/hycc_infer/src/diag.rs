use hycc_diagnostic::{
    Diagnostic, DiagnosticContext, DiagnosticCtx,
    diagnostic::{Diag, DiagNoteKind, DiagnosticKind},
};
use hycc_span::Span;

#[derive(Debug)]
pub struct InferDiagDataCtx {
    // pub interner: &'i SymbolInterner,
    // pub definitions: &'d DefinitionTable,
    // pub hir_table: &'t HirTable<'h>,
    // pub scope_ctx: &'s ScopeCtx,
}

impl InferDiagDataCtx {
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
pub struct InferDiagCtx(Vec<InferDiag>, bool);

impl InferDiagCtx {
    #[allow(unused)]
    const RESOLVER_NOTE_OFFSET: u16 = 200;
    #[allow(unused)]
    const RESOLVER_WARNING_OFFSET: u16 = 300;
    const RESOLVER_ERROR_OFFSET: u16 = 450;

    pub fn new() -> Self {
        Self(Vec::new(), false)
    }

    pub fn warning(&mut self, span: Span, kind: InferDiagWarningKind) {
        self.add(InferDiag {
            span,
            kind: InferDiagKind::Warning(kind),
        });
    }

    pub fn error(&mut self, span: Span, kind: InferDiagErrorKind) {
        self.add(InferDiag {
            span,
            kind: InferDiagKind::Error(kind),
        });
    }
}

impl DiagnosticContext<InferDiagDataCtx, InferDiag> for InferDiagCtx {
    fn data(&self) -> &Vec<InferDiag> {
        &self.0
    }

    fn data_mut(&mut self) -> &mut Vec<InferDiag> {
        &mut self.0
    }

    fn error_occurred(&self) -> bool {
        self.1
    }

    fn emit(&self, target: &mut DiagnosticCtx, ctx: InferDiagDataCtx) {
        for diag in self.data() {
            target.add(diag.emit(&ctx));
        }
    }
}

#[derive(Debug, Clone)]
pub enum InferDiagKind {
    Note(DiagNoteKind),
    Warning(InferDiagWarningKind),
    Error(InferDiagErrorKind),
}

#[derive(Debug, Clone)]
pub enum InferDiagWarningKind {}

#[derive(Debug, Clone)]
pub enum InferDiagErrorKind {}

#[derive(Debug, Clone)]
pub struct InferDiag {
    pub kind: InferDiagKind,
    pub span: Span,
}

impl InferDiag {
    pub fn warning(span: Span, kind: InferDiagWarningKind) -> Self {
        Self {
            span,
            kind: InferDiagKind::Warning(kind),
        }
    }

    pub fn error(span: Span, kind: InferDiagErrorKind) -> Self {
        Self {
            span,
            kind: InferDiagKind::Error(kind),
        }
    }
}

impl Diag<InferDiagDataCtx> for InferDiag {
    fn emit(&self, ctx: &InferDiagDataCtx) -> Diagnostic {
        use InferDiagErrorKind as Err;
        use InferDiagKind::*;

        let InferDiagDataCtx { .. } = *ctx;

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
                    let code = (unsafe { *(&self.kind as *const InferDiagKind as *const u8) })
                        as u16
                        + InferDiagCtx::RESOLVER_ERROR_OFFSET;

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
