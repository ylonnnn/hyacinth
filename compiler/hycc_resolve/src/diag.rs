use std::path::{self, PathBuf};

use hycc_diagnostic::{
    Diagnostic, DiagnosticContext, DiagnosticCtx,
    diagnostic::{Diag, DiagnosticKind},
};
use hycc_hir::{
    HirTable,
    def::{DefId, DefSpace, DefinitionTable},
    scope::ScopeCtx,
};
use hycc_session::config;
use hycc_source::SourceRegistry;
use hycc_span::Span;
use hycc_symbol::{Symbol, SymbolInterner};
use hycc_util::ternary;

pub type ResolveResult<T = (), E = ResolverDiag> = Result<T, E>;

#[derive(Debug, Clone)]
pub struct ResolverDiagDataCtx<'i, 't, 'h, 'd, 's> {
    pub interner: &'i SymbolInterner,
    pub hir_table: &'t HirTable<'h>,
    pub definitions: &'d DefinitionTable,
    pub scope_ctx: &'s ScopeCtx,
}

impl<'i, 't, 'h, 'd, 's> ResolverDiagDataCtx<'i, 't, 'h, 'd, 's> {
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
pub struct ResolverDiagCtx(Vec<ResolverDiag>, bool);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolverDiagCtxState {
    Synchronized,
    Disarray,
}

impl ResolverDiagCtx {
    #[allow(unused)]
    const RESOLVER_NOTE_OFFSET: u16 = 200;
    #[allow(unused)]
    const RESOLVER_WARNING_OFFSET: u16 = 300;
    const RESOLVER_ERROR_OFFSET: u16 = 420;

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

impl<'i, 't, 'h, 'd, 's> DiagnosticContext<ResolverDiagDataCtx<'i, 't, 'h, 'd, 's>, ResolverDiag>
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

    fn add(&mut self, diagnostic: ResolverDiag) -> Option<&mut ResolverDiag> {
        self.1 = self.1 || diagnostic.kind.is_error();
        let data = self.data_mut();

        data.push(diagnostic);
        data.last_mut()
    }

    fn emit(&self, target: &mut DiagnosticCtx, mut ctx: ResolverDiagDataCtx<'i, 't, 'h, 'd, 's>) {
        for diag in self.data() {
            target.add(diag.emit(&mut ctx));
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResolverDiagKind {
    Warning(ResolverDiagWarningKind),
    Error(ResolverDiagErrorKind),
}

#[derive(Debug, Clone)]
pub enum ResolverDiagWarningKind {}

#[derive(Debug, Clone)]
pub enum ResolverDiagErrorKind {
    Duplication { ident: Symbol, earlier_def: DefId },

    UnrecognizedSymbol(Symbol, Option<DefSpace>),
    // UnrecognizedMember { name: Symbol, ty_id: TyId },

    // InvalidPetalResolution(Symbol, DefId),
    // InvalidInference,
    InaccessibleSymbol(Symbol),
}

impl<'i, 'h, 'd, 's> ResolverDiagKind {
    pub fn is_warning(&self) -> bool {
        matches!(self, Self::Warning(..))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(..))
    }
}

#[derive(Debug, Clone)]
pub struct ResolverDiag {
    pub kind: ResolverDiagKind,
    pub span: Span,
}

impl<'i, 'h, 'd, 's> ResolverDiag {
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

impl<'i, 't, 'h, 'd, 's> Diag<ResolverDiagDataCtx<'i, 't, 'h, 'd, 's>> for ResolverDiag {
    fn emit(&self, ctx: &mut ResolverDiagDataCtx<'i, 't, 'h, 'd, 's>) -> Diagnostic {
        use ResolverDiagErrorKind as Err;
        use ResolverDiagKind::*;

        let ResolverDiagDataCtx {
            interner,
            definitions,
            ..
        } = *ctx;
        let code = (unsafe { *(&self.kind as *const ResolverDiagKind as *const u8) }) as u16
            + ResolverDiagCtx::RESOLVER_ERROR_OFFSET;

        let kind = match &self.kind {
            Warning(kind) => DiagnosticKind::Warning(match kind {
                _ => "".into(),
            }),

            Error(kind) => DiagnosticKind::Error(
                code,
                match kind {
                    Err::Duplication { ident, earlier_def } => {
                        format!(
                            "definition with the identifier `{}` already exists.",
                            interner.get(*ident)
                        )
                    }

                    Err::UnrecognizedSymbol(name, space) => {
                        format!(
                            "cannot resolved unrecognized {} `{}`",
                            space.map_or_else(|| String::from("symbol"), |space| space.to_string()),
                            interner.get(*name)
                        )
                    }

                    Err::InaccessibleSymbol(symbol) => {
                        format!(
                            "`{}` is inaccessible in this context.",
                            interner.get(*symbol)
                        )
                    }
                },
            ),
        };

        let mut diag = Diagnostic::new(self.span, kind);

        match &self.kind {
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

                _ => {}
            },
        }

        diag
    }
}
