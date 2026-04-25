use hycc_diagnostic::{
    Diagnostic, DiagnosticContext, DiagnosticCtx,
    diagnostic::{Diag, DiagNoteKind, DiagnosticKind},
};
use hycc_hir::def::{DefId, DefKind, DefinitionTable};
use hycc_span::Span;
use hycc_symbol::{Symbol, SymbolInterner};
use hycc_ty::{
    context::{TyCtx, TyId},
    fmt::TyFormatter,
    ty::Ty,
};
use hycc_util::bug;

#[derive(Debug)]
pub struct InferDiagDataCtx<'t, 'd, 'i> {
    fmt: TyFormatter<'t, 'd, 'i>,
}

impl<'t, 'd, 'i> InferDiagDataCtx<'t, 'd, 'i> {
    pub fn new(
        tctx: &'t TyCtx,
        definitions: &'d DefinitionTable,
        interner: &'i SymbolInterner,
    ) -> Self {
        Self {
            fmt: TyFormatter::new(&tctx, &definitions, &interner),
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

impl<'t, 'd, 'i> DiagnosticContext<InferDiagDataCtx<'t, 'd, 'i>, InferDiag> for InferDiagCtx {
    fn data(&self) -> &Vec<InferDiag> {
        &self.0
    }

    fn data_mut(&mut self) -> &mut Vec<InferDiag> {
        &mut self.0
    }

    fn error_occurred(&self) -> bool {
        self.1
    }

    fn emit(&self, target: &mut DiagnosticCtx, ctx: InferDiagDataCtx<'t, 'd, 'i>) {
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
pub enum InferDiagErrorKind {
    TypeMismatch {
        ann_span: Span,
        expected: TyId,
        received: TyId,
    },

    InvalidNonStructInstantiation {
        name: Symbol,
        def_id: DefId,
    },

    UnrecognizedField {
        field: Symbol,
        struct_def: DefId,
    },

    MissingFields {
        field_mask: u64,
        def_id: DefId,
    },

    FieldReinitialization {
        field: Symbol,
        earlier_span: Span,
    },

    UnresolvedTy(Ty),
}

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

impl<'t, 'd, 'i> Diag<InferDiagDataCtx<'t, 'd, 'i>> for InferDiag {
    fn emit(&self, ctx: &InferDiagDataCtx<'t, 'd, 'i>) -> Diagnostic {
        use InferDiagErrorKind as Err;
        use InferDiagKind::*;

        let InferDiagDataCtx { fmt, .. } = ctx;

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
                            Err::TypeMismatch {
                                expected, received, ..
                            } => {
                                format!(
                                    "expected type `{}`, received type `{}`.",
                                    fmt.fmt_id(*expected),
                                    fmt.fmt_id(*received)
                                )
                            }

                            Err::InvalidNonStructInstantiation { name, .. } => {
                                format!(
                                    "cannot instantiate non-struct definition `{}`.",
                                    fmt.interner.get(*name)
                                )
                            }

                            Err::UnrecognizedField { field, struct_def } => {
                                let def = fmt.definitions.get(*struct_def);
                                format!(
                                    "cannot recognize field `{}` from struct `{}`.",
                                    fmt.interner.get(*field),
                                    fmt.interner.get(def.name),
                                )
                            }

                            Err::MissingFields { field_mask, def_id } => {
                                let def = fmt.definitions.get(*def_id);
                                let DefKind::Struct(strct) = &def.kind else {
                                    unreachable!()
                                };

                                let missing_fields = strct
                                    .fields
                                    .iter()
                                    .enumerate()
                                    .filter(|(i, _)| ((field_mask >> i) & 1) == 1)
                                    .map(|(_, field)| format!("`{}`", fmt.interner.get(field.name)))
                                    .collect::<Vec<_>>()
                                    .join(", ");

                                format!(
                                    "missing fields in initializer of `{}`: {}",
                                    fmt.interner.get(def.name),
                                    missing_fields
                                )
                            }

                            Err::FieldReinitialization { field, .. } => {
                                format!(
                                    "field `{}` has already been initialized.",
                                    fmt.interner.get(*field)
                                )
                            }

                            Err::UnresolvedTy(ty) => {
                                let Ty { id, span } = ty;

                                // format!("{}", fmt.fmt_id(id))
                                format!("")
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
                Err::TypeMismatch { ann_span, .. } => {
                    if ann_span.src_id.is_valid() {
                        diag.detail(
                            *ann_span,
                            DiagnosticKind::Note(format!("expected due to the type annotation.")),
                        );
                    }
                }

                Err::InvalidNonStructInstantiation { name, def_id } => {
                    let def = fmt.definitions.get(*def_id);

                    diag.detail(
                        def.span,
                        DiagnosticKind::Note(format!(
                            "`{}` is defined here as {} `{}`",
                            fmt.interner.get(*name),
                            def.kind.article(),
                            def.kind.kind()
                        )),
                    );
                }

                Err::UnrecognizedField { struct_def, .. } => {
                    let def = fmt.definitions.get(*struct_def);
                    let DefKind::Struct(struct_def) = &def.kind else {
                        bug!("struct_def is expected to be a valid def_id of a struct definition")
                    };

                    diag.detail(
                        def.span,
                        DiagnosticKind::Note(format!(
                            "struct `{}` has the following fields: {}",
                            fmt.interner.get(def.name),
                            struct_def
                                .fields
                                .iter()
                                .map(|field| format!("`{}`", fmt.interner.get(field.name)))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )),
                    );
                }

                Err::FieldReinitialization {
                    field,
                    earlier_span,
                } => {
                    diag.detail(
                        *earlier_span,
                        DiagnosticKind::Note(format!(
                            "earlier initialization of `{}`.",
                            fmt.interner.get(*field)
                        )),
                    );
                }

                _ => {}
            },
        };

        diag
    }
}
