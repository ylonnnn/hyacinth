use std::{
    fmt::Display,
    path::{self, PathBuf},
};

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
    ctx::{TyCtx, TyId},
    fmt::TyFormatter,
};
use hycc_util::ternary;

pub type ResolveResult<T = (), E = ResolverDiag> = Result<T, E>;

impl<'c, 'h, T> FromResultEmitter<ResolverDiagCtx<'c>, ResolverDiagDataCtx<'c, 'h>, ResolverDiag, T>
    for ResolveResult<T, ResolverDiag>
{
    fn emit(self, dctx: &mut ResolverDiagCtx<'c>) -> Option<T> {
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

#[repr(u16)]
#[derive(Debug, Clone)]
pub enum ResolverDiagErrorKind {
    Duplication { ident: Symbol, earlier_def: DefId },

    UnrecognizedSymbol(Symbol, Option<DefSpace>),
    UnrecognizedMember { name: Symbol, ty_id: TyId },

    IllegalPetalTyUsage(DefId),
    InvalidInference,
    Inaccessible(Symbol, Option<SymbolKind>),

    GenericArgumentArityMismatch(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Item,
    AssocItem,
    Field,
}

impl Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Self::Item => "item",
                Self::AssocItem => "associated item",
                Self::Field => "field",
            }
        )
    }
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

        let (kind, message, extra) = match &self.kind {
            ResolverDiagKind::Info => (DiagKind::Info, "".into(), None),
            ResolverDiagKind::Warning => (DiagKind::Warning, "".into(), None),

            ResolverDiagKind::Error(kind) => {
                let (message, extra) = match kind {
                    Duplication { ident, earlier_def } => {
                        let s_ident = ctx.fmt.interner.get(*ident);
                        (
                            format!(
                                "definition with the identifier `{}` already exists",
                                s_ident
                            ),
                            Some(format!("redefinition of `{}`", s_ident)),
                        )
                    }

                    UnrecognizedSymbol(name, space) => (
                        format!(
                            "cannot resolved unrecognized {} `{}`",
                            space.map_or_else(|| String::from("symbol"), |space| space.to_string()),
                            ctx.fmt.interner.get(*name),
                        ),
                        Some(format!("could not find in this scope")),
                    ),

                    UnrecognizedMember { name, ty_id } => {
                        let s_ty = ctx.fmt.fmt_id(*ty_id);
                        (
                            format!(
                                "unrecognized associated item `{}` from `{}`",
                                ctx.fmt.interner.get(*name),
                                s_ty,
                            ),
                            Some(format!("no associated item from `{s_ty}`")),
                        )
                    }

                    IllegalPetalTyUsage(def_id) => {
                        let def = ctx.fmt.definitions.get(*def_id);
                        (
                            format!(
                                "cannot use petal `{}` as a type",
                                ctx.fmt.interner.get(def.name)
                            ),
                            None,
                        )
                    }

                    InvalidInference => (
                        format!("cannot infer type in this ctx"),
                        Some("requires known type".into()),
                    ),

                    Inaccessible(symbol, kind) => (
                        format!(
                            "{}`{}` is inaccessible in this context",
                            kind.map_or_else(|| "".into(), |kind| format!("{} ", kind)),
                            ctx.fmt.interner.get(*symbol)
                        ),
                        Some("cannot be accessed from this petal".into()),
                    ),

                    GenericArgumentArityMismatch(data) => {
                        let (expected, received) =
                            ((*data >> u8::BITS) as u8, (*data & u8::MAX as u16) as u8);
                        (
                            "generic argument arity mismatch".into(),
                            Some(format!(
                                "expected at most `{}` but received `{}`",
                                expected, received,
                            )),
                        )
                    }
                };

                (
                    DiagKind::Error(
                        hycc_util::enums::tag_of::<u16, ResolverDiagErrorKind>(&kind)
                            + ResolverDiagCtx::ERROR_CODE_OFFSET,
                    ),
                    message,
                    extra,
                )
            }
        };

        let mut diag = Diag::new_with_extra(kind, self.span, message, extra);

        match &self.kind {
            ResolverDiagKind::Error(kind) => match kind {
                Duplication { ident, earlier_def } => {
                    // TODO: add note that builtin definitions cannot be overwritten
                    let def = ctx.fmt.definitions.get(*earlier_def);

                    diag.note(
                        def.span,
                        format!("earlier definition of `{}`", ctx.fmt.interner.get(*ident)),
                    );
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
