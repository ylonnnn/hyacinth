use hycc_diagnostic::{
    Diagnostic, DiagnosticContext, DiagnosticCtx,
    diagnostic::{Diag, DiagNoteKind, DiagnosticKind},
};
use hycc_hir::{
    HirTable,
    def::{DefId, DefinitionTable},
    scope::ScopeCtx,
};
use hycc_span::Span;
use hycc_symbol::{Symbol, SymbolInterner};

#[derive(Debug)]
pub struct CollectorDiagDataCtx<'i, 't, 'h, 'd, 's> {
    pub interner: &'i SymbolInterner,
    pub hir_table: &'t HirTable<'h>,
    pub definitions: &'d DefinitionTable,
    pub scope_ctx: &'s ScopeCtx,
}

impl<'i, 't, 'h, 'd, 's> CollectorDiagDataCtx<'i, 't, 'h, 'd, 's> {
    pub fn new(
        interner: &'i SymbolInterner,
        hir_table: &'t HirTable<'h>,
        definitions: &'d DefinitionTable,
        scope_ctx: &'s ScopeCtx,
    ) -> Self {
        Self {
            interner,
            hir_table,
            definitions,
            scope_ctx,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CollectorDiagCtx(Vec<CollectorDiag>, bool);

impl CollectorDiagCtx {
    #[allow(unused)]
    const COLLECTOR_NOTE_OFFSET: u16 = 200;
    #[allow(unused)]
    const COLLECTOR_WARNING_OFFSET: u16 = 300;
    const COLLECTOR_ERROR_OFFSET: u16 = 440;

    pub fn new() -> Self {
        Self(Vec::new(), false)
    }

    pub fn warning(&mut self, span: Span, kind: CollectorDiagWarningKind) {
        self.add(CollectorDiag {
            span,
            kind: CollectorDiagKind::Warning(kind),
        });
    }

    pub fn error(&mut self, span: Span, kind: CollectorDiagErrorKind) {
        self.add(CollectorDiag {
            span,
            kind: CollectorDiagKind::Error(kind),
        });
    }
}

impl<'i, 't, 'h, 'd, 's> DiagnosticContext<CollectorDiagDataCtx<'i, 't, 'h, 'd, 's>, CollectorDiag>
    for CollectorDiagCtx
{
    fn data(&self) -> &Vec<CollectorDiag> {
        &self.0
    }

    fn data_mut(&mut self) -> &mut Vec<CollectorDiag> {
        &mut self.0
    }

    fn error_occurred(&self) -> bool {
        self.1
    }

    fn emit(&self, target: &mut DiagnosticCtx, ctx: CollectorDiagDataCtx<'i, 't, 'h, 'd, 's>) {
        for diag in self.data() {
            target.add(diag.emit(&ctx));
        }
    }
}

#[derive(Debug, Clone)]
pub enum CollectorDiagKind {
    Note(DiagNoteKind),
    Warning(CollectorDiagWarningKind),
    Error(CollectorDiagErrorKind),
}

#[derive(Debug, Clone)]
pub enum CollectorDiagWarningKind {}

#[derive(Debug, Clone)]
pub enum CollectorDiagErrorKind {
    Duplication { ident: Symbol, earlier_def: DefId },
}

#[derive(Debug, Clone)]
pub struct CollectorDiag {
    pub kind: CollectorDiagKind,
    pub span: Span,
}

impl CollectorDiag {
    pub fn warning(span: Span, kind: CollectorDiagWarningKind) -> Self {
        Self {
            span,
            kind: CollectorDiagKind::Warning(kind),
        }
    }

    pub fn error(span: Span, kind: CollectorDiagErrorKind) -> Self {
        Self {
            span,
            kind: CollectorDiagKind::Error(kind),
        }
    }
}

impl<'i, 't, 'h, 'd, 's> Diag<CollectorDiagDataCtx<'i, 't, 'h, 'd, 's>> for CollectorDiag {
    fn emit(&self, ctx: &CollectorDiagDataCtx<'i, 't, 'h, 'd, 's>) -> Diagnostic {
        use CollectorDiagErrorKind as Err;
        use CollectorDiagKind::*;

        let CollectorDiagDataCtx {
            interner,
            definitions,
            ..
        } = *ctx;
        let code = (unsafe { *(&self.kind as *const CollectorDiagKind as *const u8) }) as u16
            + CollectorDiagCtx::COLLECTOR_ERROR_OFFSET;

        let mut diag = Diagnostic::new(
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
                        Err::Duplication {
                            ident,
                            earlier_def: _,
                        } => {
                            format!(
                                "identifier `{}` has already been used in an earlier definition.",
                                interner.get(*ident)
                            )
                        }
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
                Err::Duplication { ident, earlier_def } => {
                    let def = definitions.get(*earlier_def);

                    if def.span.src_id.is_valid() {
                        diag.detail(
                            def.span,
                            DiagnosticKind::Note(format!(
                                "earlier definition of `{}`",
                                interner.get(*ident)
                            )),
                        );
                    }

                    // TODO: add note that builtin definitions cannot be overwritten
                }

                #[allow(unreachable_patterns)]
                _ => {}
            },
        }

        diag
    }
}
