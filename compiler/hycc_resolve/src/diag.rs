use std::path::{self, PathBuf};

use hycc_diagnostic::diagnostic::{
    Diag, DiagCtx, DiagEmitter, DiagKind, DiagLike, Diagnostics, FromResultEmitter,
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
use hycc_ty::{
    context::{TyCtx, TyId},
    fmt::TyFormatter,
};
use hycc_util::ternary;

pub type ResolveResult<T = (), E = ResolverDiag> = Result<T, E>;

impl<'c, 'h, RT>
    FromResultEmitter<ResolverDiagCtx<'c>, ResolverDiagDataCtx<'c, 'h>, ResolverDiag, RT>
    for ResolveResult<RT, ResolverDiag>
{
    fn emit(self, dctx: &mut ResolverDiagCtx<'c>) -> Option<RT> {
        match self {
            Ok(val) => Some(val),
            Err(diag) => {
                dctx.add(diag);
                None
            }
        }
    }
}

#[derive(Debug)]
pub struct ResolverDiagDataCtx<'c, 'h> {
    pub fmt: TyFormatter<'c>,
    pub hir_table: &'c HirTable<'h>,
    pub scope_ctx: &'c ScopeCtx,
}

impl<'c, 'h> ResolverDiagDataCtx<'c, 'h> {
    pub fn new(
        tctx: &'c mut TyCtx,
        interner: &'c SymbolInterner,
        hir_table: &'c HirTable<'h>,
        definitions: &'c DefinitionTable,
        scope_ctx: &'c ScopeCtx,
    ) -> Self {
        Self {
            fmt: TyFormatter::new(tctx, definitions, interner),
            hir_table,
            scope_ctx,
        }
    }
}

#[derive(Debug)]
pub struct ResolverDiagCtx<'c>(Vec<ResolverDiag>, &'c mut DiagCtx, bool);

impl<'c> ResolverDiagCtx<'c> {
    pub fn new(dctx: &'c mut DiagCtx) -> Self {
        Self(Vec::new(), dctx, false)
    }

    pub fn error(&mut self, span: Span, kind: ResolverDiagErrorKind) {
        self.add(ResolverDiag {
            span,
            kind: ResolverDiagKind::Error(kind),
        });
    }
}

impl<'c, 'h> Diagnostics<ResolverDiagDataCtx<'c, 'h>, ResolverDiag> for ResolverDiagCtx<'c> {
    const ERROR_CODE_OFFSET: u16 = 400;

    fn data(&self) -> &[ResolverDiag] {
        &self.0
    }

    fn data_mut(&mut self) -> &mut Vec<ResolverDiag> {
        &mut self.0
    }

    fn error_flag(&mut self) -> &mut bool {
        &mut self.2
    }

    fn emit(&mut self, mut ctx: ResolverDiagDataCtx<'c, 'h>) {
        for diag in &self.0 {
            self.1.add(diag.emit(&mut ctx));
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResolverDiagKind {
    Info,
    Warning,
    Error(ResolverDiagErrorKind),
}

#[derive(Debug, Clone)]
pub enum ResolverDiagWarningKind {}

#[derive(Debug, Clone)]
pub enum ResolverDiagErrorKind {
    Duplication { ident: Symbol, earlier_def: DefId },

    UnrecognizedSymbol(Symbol, Option<DefSpace>),
    UnrecognizedMember { name: Symbol, ty_id: TyId },

    IllegalPetalTyUsage(DefId),
    InvalidInference,
    InaccessibleSymbol(Symbol),

    GenericArgumentArityMismatch(u16),
}

#[derive(Debug, Clone)]
pub struct ResolverDiag {
    pub kind: ResolverDiagKind,
    pub span: Span,
}

impl ResolverDiag {
    pub fn error(span: Span, kind: ResolverDiagErrorKind) -> Self {
        Self {
            span,
            kind: ResolverDiagKind::Error(kind),
        }
    }
}

impl DiagLike for ResolverDiag {
    fn is_info(&self) -> bool {
        matches!(&self.kind, ResolverDiagKind::Info)
    }

    fn is_warning(&self) -> bool {
        matches!(&self.kind, ResolverDiagKind::Warning)
    }

    fn is_error(&self) -> bool {
        matches!(&self.kind, ResolverDiagKind::Error(_))
    }
}

impl<'c, 'h> DiagEmitter<ResolverDiagDataCtx<'c, 'h>> for ResolverDiag {
    fn emit(&self, ctx: &mut ResolverDiagDataCtx<'c, 'h>) -> Diag {
        use ResolverDiagErrorKind::*;

        let (kind, message) = match &self.kind {
            ResolverDiagKind::Info => (DiagKind::Info, "".into()),
            ResolverDiagKind::Warning => (DiagKind::Warning, "".into()),

            ResolverDiagKind::Error(kind) => {
                let message = match kind {
                    Duplication { ident, earlier_def } => {
                        format!(
                            "definition with the identifier `{}` already exists.",
                            ctx.fmt.interner.get(*ident)
                        )
                    }

                    UnrecognizedSymbol(name, space) => {
                        format!(
                            "cannot resolved unrecognized {} `{}`",
                            space.map_or_else(|| String::from("symbol"), |space| space.to_string()),
                            ctx.fmt.interner.get(*name)
                        )
                    }

                    UnrecognizedMember { name, ty_id } => {
                        format!(
                            "cannot recognize associated item `{}` from type `{}`.",
                            ctx.fmt.interner.get(*name),
                            ctx.fmt.fmt_id(*ty_id)
                        )
                    }

                    IllegalPetalTyUsage(def_id) => {
                        let def = ctx.fmt.definitions.get(*def_id);
                        format!(
                            "cannot use petal `{}` as a type.",
                            ctx.fmt.interner.get(def.name)
                        )
                    }

                    InvalidInference => format!("cannot infer type in this context."),

                    InaccessibleSymbol(symbol) => {
                        format!(
                            "`{}` is inaccessible in this context.",
                            ctx.fmt.interner.get(*symbol)
                        )
                    }

                    GenericArgumentArityMismatch(data) => {
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
                };

                (
                    DiagKind::Error(
                        hycc_util::enums::tag_of::<u16, _>(&kind)
                            + ResolverDiagCtx::ERROR_CODE_OFFSET,
                    ),
                    message,
                )
            }
        };

        let mut diag = Diag::new(kind, self.span, message);

        match &self.kind {
            ResolverDiagKind::Error(kind) => match kind {
                Duplication { ident, earlier_def } => {
                    // let def = definitions.get(*earlier_def);

                    // TODO: add note that builtin definitions cannot be overwritten

                    // if def.span.src_id.is_valid() {
                    //     diag.detail(
                    //         def.span,
                    //         DiagKind::Note(format!(
                    //             "earlier definition of `{}`",
                    //             interner.get(*ident)
                    //         )),
                    //     );
                    // }
                }

                IllegalPetalTyUsage(def_id) => {
                    // TODO: add note and/or sub-diagnostic pointing to
                    // the definition of the petal
                }

                _ => {}
            },

            _ => {}
        }

        diag
    }
}
