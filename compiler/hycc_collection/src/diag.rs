use hycc_diagnostic::{
    Diagnostic, DiagnosticContext, DiagnosticCtx,
    diagnostic::{Diag, DiagnosticKind},
};
use hycc_hir::def::DefId;
use hycc_span::Span;
use hycc_symbol::{Symbol, SymbolInterner};

#[derive(Debug, Clone)]
pub struct CollectorDiagDataCtx<'i> {
    pub interner: &'i SymbolInterner,
    // pub definitions: &
}

impl<'i> CollectorDiagDataCtx<'i> {
    pub fn new(interner: &'i SymbolInterner) -> Self {
        Self { interner }
    }
}

#[derive(Debug, Clone)]
pub struct CollectorDiagCtx(Vec<CollectorDiag>, bool);

impl CollectorDiagCtx {
    #[allow(unused)]
    const COLLECTOR_NOTE_OFFSET: u16 = 200;
    #[allow(unused)]
    const COLLECTOR_WARNING_OFFSET: u16 = 300;
    const COLLECTOR_ERROR_OFFSET: u16 = 400;

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

impl<'i> DiagnosticContext<CollectorDiagDataCtx<'i>, CollectorDiag> for CollectorDiagCtx {
    fn data(&self) -> &Vec<CollectorDiag> {
        &self.0
    }

    fn data_mut(&mut self) -> &mut Vec<CollectorDiag> {
        &mut self.0
    }

    fn error_occurred(&self) -> bool {
        self.1
    }

    fn emit(&self, target: &mut DiagnosticCtx, ctx: CollectorDiagDataCtx<'i>) {
        for diag in self.data() {
            target.add(diag.emit(&ctx));
        }
    }
}

#[derive(Debug, Clone)]
pub enum CollectorDiagKind {
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

impl<'i> Diag<CollectorDiagDataCtx<'i>> for CollectorDiag {
    fn emit(&self, ctx: &CollectorDiagDataCtx<'i>) -> Diagnostic {
        use CollectorDiagErrorKind as Err;
        use CollectorDiagKind::*;

        let CollectorDiagDataCtx { interner } = *ctx;
        let code = (unsafe { *(&self.kind as *const CollectorDiagKind as *const u8) }) as u16
            + CollectorDiagCtx::COLLECTOR_ERROR_OFFSET;

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
        };

        Diagnostic::new(self.span, kind)
    }
}
