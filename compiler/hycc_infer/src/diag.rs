use hycc_diagnostic::{
    Diagnostic, DiagnosticContext, DiagnosticCtx,
    diagnostic::{Diag, DiagNoteKind, DiagnosticKind},
};
use hycc_hir::{
    def::{AdtKind, DefId, DefKind, DefinitionTable},
    expr::HirFieldAccessFieldKind,
};
use hycc_span::Span;
use hycc_symbol::{Symbol, SymbolInterner};
use hycc_ty::{
    context::{TyCtx, TyId},
    fmt::TyFormatter,
    ty::{AccessKind, RefMutability, Ty},
};
use hycc_util::{bug, ternary};

#[derive(Debug)]
pub struct InferDiagDataCtx<'t, 'd, 'i> {
    fmt: TyFormatter<'t, 'd, 'i>,
}

impl<'t, 'd, 'i> InferDiagDataCtx<'t, 'd, 'i> {
    pub fn new(
        tctx: &'t mut TyCtx,
        definitions: &'d DefinitionTable,
        interner: &'i SymbolInterner,
    ) -> Self {
        Self {
            fmt: TyFormatter::new(tctx, &definitions, &interner),
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
    fn add(&mut self, diagnostic: InferDiag) -> Option<&mut InferDiag> {
        self.1 = self.1 || matches!(&diagnostic.kind, InferDiagKind::Error(_));
        let data = self.data_mut();

        data.push(diagnostic);
        data.last_mut()
    }

    fn data(&self) -> &Vec<InferDiag> {
        &self.0
    }

    fn data_mut(&mut self) -> &mut Vec<InferDiag> {
        &mut self.0
    }

    fn error_occurred(&self) -> bool {
        self.1
    }

    fn emit(&self, target: &mut DiagnosticCtx, mut ctx: InferDiagDataCtx<'t, 'd, 'i>) {
        for diag in self.data() {
            target.add(diag.emit(&mut ctx));
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
        expectation_span: Span,
        expected: TyId,
        received: TyId,
    },

    InvalidNonStructInstantiation {
        name: Symbol,
        def_id: DefId,
    },

    UnrecognizedField {
        field: HirFieldAccessFieldKind,
        ty_id: TyId,
    },

    UnrecognizedMethod {
        method: Symbol,
        ty_id: TyId,
    },

    UnrecognizedFieldInitialization {
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

    IllegalInvocation(TyId),
    ArgumentArityMismatch(u16), // expected: 8-bits | received: 8-bits
    GenericArgumentArityMismatch(u16), // expected: 8-bits | received: 8-bits

    MissingElseBranch,

    InaccessibleMember {
        name: Symbol,
        kind: MemberKind,
    },

    InvalidAssocFnInvocation {
        name: Symbol,
        def_id: DefId,
        ty_id: TyId,
    },

    ReceiverAccessMismatch {
        access: AccessKind,
        requested: AccessKind,
        call: (Symbol, Span),
        def_id: DefId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemberKind {
    AssocFn,
    Field,
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
    fn emit(&self, ctx: &mut InferDiagDataCtx<'t, 'd, 'i>) -> Diagnostic {
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

                            Err::UnrecognizedField { field, ty_id } => {
                                format!(
                                    "cannot recognize field `{}` from type `{}`.",
                                    match &field {
                                        HirFieldAccessFieldKind::Ident(ident) =>
                                            fmt.interner.get(*ident).into(),
                                        HirFieldAccessFieldKind::Index(idx) => idx.to_string(),
                                    },
                                    fmt.fmt_id(*ty_id),
                                )
                            }

                            Err::UnrecognizedMethod { method, ty_id } => {
                                format!(
                                    "cannot recognize method `{}` from type `{}`.",
                                    fmt.interner.get(*method),
                                    fmt.fmt_id(*ty_id),
                                )
                            }

                            Err::UnrecognizedFieldInitialization { field, struct_def } => {
                                let def = fmt.definitions.get(*struct_def);
                                format!(
                                    "cannot recognize field `{}` from struct `{}`.",
                                    fmt.interner.get(*field),
                                    fmt.interner.get(def.name),
                                )
                            }

                            Err::MissingFields { field_mask, def_id } => {
                                let def = fmt.definitions.get(*def_id);
                                let adt_def = def.kind.expect_adt();

                                let strct = adt_def.expect_struct();
                                let missing_fields = strct
                                    .fields
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(i, field)| {
                                        ternary!(
                                            (field_mask >> i) & 1 != 1,
                                            None,
                                            Some(format!("`{}`", fmt.interner.get(field.name)))
                                        )
                                    })
                                    .collect::<Vec<_>>();

                                format!(
                                    "missing field{} in initializer of `{}`: {}",
                                    ternary!(missing_fields.len() > 1, "s", ""),
                                    fmt.interner.get(def.name),
                                    missing_fields.join(", ")
                                )
                            }

                            Err::FieldReinitialization { field, .. } => {
                                format!(
                                    "field `{}` has already been initialized.",
                                    fmt.interner.get(*field)
                                )
                            }

                            Err::UnresolvedTy(ty) => {
                                let Ty { id, .. } = ty;

                                format!("unresolved type `{}`.", fmt.fmt_id(*id))
                            }

                            Err::IllegalInvocation(ty_id) => {
                                format!(
                                    "cannot invoke expression of type `{}`.",
                                    fmt.fmt_id(*ty_id)
                                )
                            }

                            Err::ArgumentArityMismatch(data) => {
                                let (expected, received) =
                                    ((*data >> u8::BITS) as u8, (*data & u8::MAX as u16) as u8);
                                format!(
                                    "expected `{}` argument{}, received `{}` argument{}.",
                                    expected,
                                    ternary!(expected == 1, "", "s"),
                                    received,
                                    ternary!(received == 1, "", "s"),
                                )
                            }

                            Err::GenericArgumentArityMismatch(data) => {
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

                            Err::MissingElseBranch => {
                                format!(
                                    "`if` expression with a non-unit consequent requires an `else` branch."
                                )
                            }

                            Err::InaccessibleMember { name, kind } => {
                                format!(
                                    "{} `{}` is inaccessible in this context.",
                                    match &kind {
                                        MemberKind::Field => "field",
                                        MemberKind::AssocFn => "associated function",
                                    },
                                    fmt.interner.get(*name)
                                )
                            }

                            Err::InvalidAssocFnInvocation { name, ty_id, .. } => {
                                format!(
                                    "cannot invoke associated function `{}::{}` through method calls.",
                                    fmt.fmt_id(*ty_id),
                                    fmt.interner.get(*name)
                                )
                            }

                            Err::ReceiverAccessMismatch {
                                access, requested, ..
                            } => {
                                format!(
                                    "cannot {} a `{}`.",
                                    match &requested {
                                        AccessKind::Owned => "move out of",
                                        AccessKind::Ref(mutability) => match &mutability {
                                            RefMutability::Immutable => "borrow",
                                            RefMutability::Mutable => "mutably borrow",
                                        },
                                    },
                                    access
                                )
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
                Err::TypeMismatch {
                    expectation_span: ann_span,
                    ..
                } => {
                    diag.detail(
                        *ann_span,
                        DiagnosticKind::Note(format!("expected due to this.")),
                    );
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

                Err::UnrecognizedFieldInitialization { struct_def, .. } => {
                    let def = fmt.definitions.get(*struct_def);
                    let adt_def = def.kind.expect_adt();

                    let struct_def = adt_def.expect_struct();
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

                Err::UnresolvedTy(ty) => {
                    diag.detail(
                        ty.span,
                        DiagnosticKind::Note(String::from(
                            "requires `type annotation` or be used in a context with `known type`.",
                        )),
                    );
                }

                Err::MissingElseBranch => {
                    diag.detail(
                        diag.span,
                        DiagnosticKind::Note(String::from(
                            "`if` may be missing its `else` branch.",
                        )),
                    );
                }

                Err::InvalidAssocFnInvocation { name, def_id, .. } => {
                    let def = fmt.definitions.get(*def_id);
                    let fn_def = def.kind.expect_fn();

                    if fn_def.params.len() < 1 {
                        diag.detail(
                        def.span,
                        DiagnosticKind::Note(format!(
                            "associated function `{}` does not have a receiving parameter compatible to type `Self`.",
                            fmt.interner.get(*name)
                        )),
                    );
                    } else {
                        let rec_param_def = fmt.definitions.get(fn_def.params[0]);
                        diag.detail(
                            rec_param_def.span,
                            DiagnosticKind::Note(format!(
                                    "receiving parameter `{}` of `{}` does not have a compatible type to `Self`.",
                                    fmt.interner.get(rec_param_def.name),
                                    fmt.interner.get(*name)
                            )),
                        );
                    }
                }

                Err::ReceiverAccessMismatch {
                    requested,
                    def_id,
                    call: method,
                    ..
                } => {
                    let def = fmt.definitions.get(*def_id);
                    let fn_def = def.kind.expect_fn();

                    diag.detail(
                        method.1,
                        DiagnosticKind::Note(format!(
                            "{} occurs due to call to `{}`.",
                            match &requested {
                                AccessKind::Owned => "move",
                                AccessKind::Ref(mutability) => match &mutability {
                                    RefMutability::Immutable => "borrow",
                                    RefMutability::Mutable => "mutable borrow",
                                },
                            },
                            fmt.interner.get(method.0)
                        )),
                    );

                    let param_def = fmt.definitions.get(fn_def.params[0]);
                    diag.detail(
                        def.span,
                        DiagnosticKind::Note(format!(
                            "`{}` is defined where the receiver `{}` must be {}.",
                            fmt.interner.get(method.0),
                            fmt.interner.get(param_def.name),
                            format!(
                                "{}`{}`",
                                ternary!(matches!(requested, AccessKind::Owned), "", "a "),
                                requested.to_string()
                            )
                        )),
                    );
                }

                _ => {}
            },
        };

        diag
    }
}
