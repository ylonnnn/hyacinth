use hycc_diagnostic::diagnostic::{
    Diag, DiagCtx, DiagEmitter, DiagKind, DiagLike, Diagnostics, FromResultEmitter,
};
use hycc_hir::{
    def::{AdtKind, DefId, DefKind, DefinitionTable},
    expr::HirFieldAccessFieldKind,
};
use hycc_resolve::diag::ResolverDiagDataCtx;
use hycc_span::Span;
use hycc_symbol::{Symbol, SymbolInterner};
use hycc_ty::{
    ctx::{TyCtx, TyId},
    fmt::TyFormatter,
    ty::{AccessKind, RefMutability, Ty},
};
use hycc_util::{bug, ternary};

pub type InferResult<T = (), E = InferDiag> = Result<T, E>;

impl<'c, T> FromResultEmitter<InferDiagCtx<'c>, InferDiagDataCtx<'c>, InferDiag, T>
    for InferResult<T, InferDiag>
{
    fn emit(self, dctx: &mut InferDiagCtx<'c>) -> Option<T> {
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
pub struct InferDiagDataCtx<'c> {
    fmt: TyFormatter<'c>,
}

impl<'c> InferDiagDataCtx<'c> {
    pub fn new(
        tctx: &'c mut TyCtx,
        definitions: &'c DefinitionTable,
        interner: &'c SymbolInterner,
    ) -> Self {
        Self {
            fmt: TyFormatter::new(tctx, &definitions, &interner),
        }
    }
}

#[derive(Debug)]
pub struct InferDiagCtx<'c>(Vec<InferDiag>, &'c mut DiagCtx, bool);

impl<'c> InferDiagCtx<'c> {
    pub fn new(dctx: &'c mut DiagCtx) -> Self {
        Self(Vec::new(), dctx, false)
    }

    pub fn error(&mut self, span: Span, kind: InferDiagErrorKind) {
        self.add(InferDiag {
            span,
            kind: InferDiagKind::Error(kind),
        });
    }
}

impl<'c> Diagnostics<InferDiagDataCtx<'c>, InferDiag> for InferDiagCtx<'c> {
    const ERROR_CODE_OFFSET: u16 = 500;

    fn data(&self) -> &[InferDiag] {
        &self.0
    }

    fn data_mut(&mut self) -> &mut Vec<InferDiag> {
        &mut self.0
    }

    fn error_flag(&mut self) -> &mut bool {
        &mut self.2
    }

    fn emit(&mut self, mut ctx: InferDiagDataCtx<'c>) {
        for diag in &self.0 {
            self.1.add(diag.emit(&mut ctx));
        }
    }
}

#[derive(Debug, Clone)]
pub enum InferDiagKind {
    Info,
    Warning,
    Error(InferDiagErrorKind),
}

#[repr(u16)]
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
    InvalidInference(TyId),
    TyComputationCycle(Span),

    IllegalInvocation(TyId),
    ArgumentArityMismatch(u16), // expected: 8-bits | received: 8-bits
    GenericArgumentArityMismatch(u16), // expected: 8-bits | received: 8-bits

    MissingElseBranch,

    UnrecognizedMember {
        name: Symbol,
        ty_id: TyId,
    },

    InaccessibleMember {
        name: Symbol,
        kind: MemberKind,
    },

    IllegalAssocFnInvocation {
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
    pub fn error(span: Span, kind: InferDiagErrorKind) -> Self {
        Self {
            span,
            kind: InferDiagKind::Error(kind),
        }
    }
}

impl DiagLike for InferDiag {
    fn is_info(&self) -> bool {
        matches!(&self.kind, InferDiagKind::Info)
    }

    fn is_warning(&self) -> bool {
        matches!(&self.kind, InferDiagKind::Warning)
    }

    fn is_error(&self) -> bool {
        matches!(&self.kind, InferDiagKind::Error(_))
    }
}

impl<'c> DiagEmitter<InferDiagDataCtx<'c>> for InferDiag {
    fn emit(&self, ctx: &mut InferDiagDataCtx<'c>) -> hycc_diagnostic::diagnostic::Diag {
        use InferDiagErrorKind::*;

        let (kind, message, extra) = match &self.kind {
            InferDiagKind::Info => (DiagKind::Info, "".into(), None),
            InferDiagKind::Warning => (DiagKind::Warning, "".into(), None),

            InferDiagKind::Error(kind) => {
                let (message, extra) = match kind {
                    TypeMismatch {
                        expected, received, ..
                    } => (
                        "mismatched types".into(),
                        Some(format!(
                            "expected `{}`, received `{}`",
                            ctx.fmt.fmt_id(*expected),
                            ctx.fmt.fmt_id(*received)
                        )),
                    ),

                    InvalidNonStructInstantiation { name, def_id } => {
                        let s_name = ctx.fmt.interner.get(*name);
                        let def = ctx.fmt.definitions.get(*def_id);

                        (
                            format!("cannot instantiate non-struct definition `{s_name}`"),
                            Some(format!(
                                "`{s_name}` is defined as {} `{}`",
                                def.kind.article(),
                                def.kind.kind()
                            )),
                        )
                    }

                    UnrecognizedField { field, ty_id } => {
                        let s_ty = ctx.fmt.fmt_id(*ty_id);
                        (
                            format!(
                                "unrecognized field `{}` from `{s_ty}`",
                                match &field {
                                    HirFieldAccessFieldKind::Ident(ident) =>
                                        ctx.fmt.interner.get(*ident).into(),
                                    HirFieldAccessFieldKind::Index(idx) => idx.to_string(),
                                },
                            ),
                            Some(format!("no field from `{s_ty}`")),
                        )
                    }

                    UnrecognizedMethod { method, ty_id } => {
                        let s_ty = ctx.fmt.fmt_id(*ty_id);
                        (
                            format!(
                                "unrecognized method `{}` from `{}`",
                                ctx.fmt.interner.get(*method),
                                s_ty,
                            ),
                            Some(format!("no method from `{s_ty}`")),
                        )
                    }

                    UnrecognizedFieldInitialization { field, struct_def } => {
                        let def = ctx.fmt.definitions.get(*struct_def);
                        (
                            format!(
                                "unrecognized field `{}` from struct `{}`",
                                ctx.fmt.interner.get(*field),
                                ctx.fmt.interner.get(def.name),
                            ),
                            Some("cannot initialize unrecognized field".into()),
                        )
                    }

                    MissingFields { field_mask, def_id } => {
                        let def = ctx.fmt.definitions.get(*def_id);
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
                                    Some(format!("`{}`", ctx.fmt.interner.get(field.name)))
                                )
                            })
                            .collect::<Vec<_>>();

                        (
                            format!(
                                "missing field{} in initializer of `{}`",
                                ternary!(missing_fields.len() > 1, "s", ""),
                                ctx.fmt.interner.get(def.name),
                            ),
                            Some(format!(
                                "missing {}",
                                hycc_util::text::list_enumeration(&missing_fields)
                            )),
                        )
                    }

                    FieldReinitialization { field, .. } => (
                        format!(
                            "field `{}` has already been initialized",
                            ctx.fmt.interner.get(*field)
                        ),
                        None,
                    ),

                    UnresolvedTy(ty) => (
                        format!("cannot resolve type `{}`", ctx.fmt.fmt_id(ty.id)),
                        Some("type annotation required".into()),
                    ),

                    InvalidInference(ty_id) => (
                        "cannot infer type at this position".into(),
                        Some(ternary!(
                            ctx.fmt.tctx.is_inferred(*ty_id),
                            "inferred types are not allowed here".into(),
                            "correct concrete type is required".into()
                        )),
                    ),

                    TyComputationCycle(_) => (
                        "type computation cycle detected".into(),
                        Some("detected a cycle in type dependency".into()),
                    ),

                    IllegalInvocation(ty_id) => (
                        "cannot invoke expression".into(),
                        Some(format!(
                            "expression is of type `{}`",
                            ctx.fmt.fmt_id(*ty_id)
                        )),
                    ),

                    ArgumentArityMismatch(data) => {
                        let (expected, received) =
                            ((*data >> u8::BITS) as u8, (*data & u8::MAX as u16) as u8);
                        (
                            format!(
                                "expected `{}` argument{}, received `{}` argument{}",
                                expected,
                                ternary!(expected == 1, "", "s"),
                                received,
                                ternary!(received == 1, "", "s"),
                            ),
                            None,
                        )
                    }

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

                    MissingElseBranch => (
                        format!(
                            "`if` expression with a non-unit consequent requires an `else` branch"
                        ),
                        None,
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

                    InaccessibleMember { name, kind } => {
                        let s_kind = match &kind {
                            MemberKind::Field => "field",
                            MemberKind::AssocFn => "associated function",
                        };
                        (
                            format!(
                                "{s_kind} `{}` is inaccessible in this ctx",
                                ctx.fmt.interner.get(*name)
                            ),
                            Some(format!("cannot access {s_kind}")),
                        )
                    }

                    IllegalAssocFnInvocation { name, ty_id, .. } => (
                        format!(
                            "cannot invoke associated function `{}::{}`",
                            ctx.fmt.fmt_id(*ty_id),
                            ctx.fmt.interner.get(*name)
                        ),
                        Some("cannot invoke as method call".into()),
                    ),

                    ReceiverAccessMismatch {
                        access, requested, ..
                    } => (
                        format!(
                            "cannot {} a `{}`",
                            match &requested {
                                AccessKind::Owned => "move out of",
                                AccessKind::Ref(mutability) => match &mutability {
                                    RefMutability::Immutable => "borrow",
                                    RefMutability::Mutable => "mutably borrow",
                                },
                            },
                            access
                        ),
                        None,
                    ),
                };

                (
                    DiagKind::Error(
                        hycc_util::enums::tag_of::<u16, InferDiagErrorKind>(&kind)
                            + InferDiagCtx::ERROR_CODE_OFFSET,
                    ),
                    message,
                    extra,
                )
            }
        };

        let mut diag = Diag::new_with_extra(kind, self.span, message, extra);

        match &self.kind {
            InferDiagKind::Error(kind) => match kind {
                TypeMismatch {
                    expectation_span: ann_span,
                    ..
                } => {
                    diag.note(*ann_span, format!("expected due to this"));
                }

                InvalidNonStructInstantiation { name, def_id } => {
                    let def = ctx.fmt.definitions.get(*def_id);

                    diag.note(
                        def.span,
                        format!(
                            "`{}` is defined here as {} `{}`",
                            ctx.fmt.interner.get(*name),
                            def.kind.article(),
                            def.kind.kind()
                        ),
                    );
                }

                UnrecognizedFieldInitialization { struct_def, .. } => {
                    let def = ctx.fmt.definitions.get(*struct_def);
                    let adt_def = def.kind.expect_adt();

                    let struct_def = adt_def.expect_struct();
                    diag.note(
                        def.span,
                        format!(
                            "struct `{}` has the following fields: {}",
                            ctx.fmt.interner.get(def.name),
                            hycc_util::text::list_enumeration(
                                &struct_def
                                    .fields
                                    .iter()
                                    .map(|field| format!("`{}`", ctx.fmt.interner.get(field.name)))
                                    .collect::<Vec<_>>()
                            )
                        ),
                    );
                }

                FieldReinitialization {
                    field,
                    earlier_span,
                } => {
                    diag.note(
                        *earlier_span,
                        format!(
                            "earlier initialization of `{}`",
                            ctx.fmt.interner.get(*field)
                        ),
                    );
                }

                UnresolvedTy(ty) => {
                    diag.add_sub_diagnostic(Diag::new_with_extra(
                        DiagKind::Info,
                        ty.span,
                        "requires context with known type",
                        Some("type annotation or usage in a context with known type is needed"),
                    ));
                }

                InvalidInference(ty_id) => {
                    if !ctx.fmt.tctx.is_inferred(*ty_id) {
                        diag.help(
                            diag.span,
                            format!(
                                "replace with the correct type: `{}`",
                                ctx.fmt.fmt_id(*ty_id)
                            ),
                        );
                    }
                }

                TyComputationCycle(span) => {
                    diag.note(*span, "cycle detected within this definition");
                }

                MissingElseBranch => {
                    diag.note(diag.span, "`if` may be missing its `else` branch");
                }

                IllegalAssocFnInvocation { name, def_id, .. } => {
                    let def = ctx.fmt.definitions.get(*def_id);
                    let fn_def = def.kind.expect_fn();

                    if fn_def.params.len() < 1 {
                        diag.note(
                        def.span,
                        format!(
                            "associated function `{}` does not have a receiving parameter compatible to type `Self`",
                            ctx.fmt.interner.get(*name)
                        ),
                    );
                    } else {
                        let rec_param_def = ctx.fmt.definitions.get(fn_def.params[0]);
                        diag.note(
                            rec_param_def.span,
                            format!(
                                "receiving parameter `{}` of `{}` does not have a compatible type to `Self`",
                                ctx.fmt.interner.get(rec_param_def.name),
                                ctx.fmt.interner.get(*name)
                            ),
                        );
                    }
                }

                ReceiverAccessMismatch {
                    requested,
                    def_id,
                    call: method,
                    ..
                } => {
                    let def = ctx.fmt.definitions.get(*def_id);
                    let fn_def = def.kind.expect_fn();

                    diag.note(
                        method.1,
                        format!(
                            "{} occurs due to call to `{}`",
                            match &requested {
                                AccessKind::Owned => "move",
                                AccessKind::Ref(mutability) => match &mutability {
                                    RefMutability::Immutable => "borrow",
                                    RefMutability::Mutable => "mutable borrow",
                                },
                            },
                            ctx.fmt.interner.get(method.0)
                        ),
                    );

                    let param_def = ctx.fmt.definitions.get(fn_def.params[0]);
                    diag.note(
                        def.span,
                        format!(
                            "`{}` is defined where the receiver `{}` must be {}",
                            ctx.fmt.interner.get(method.0),
                            ctx.fmt.interner.get(param_def.name),
                            format!(
                                "{}`{}`",
                                ternary!(matches!(requested, AccessKind::Owned), "", "a "),
                                requested.to_string()
                            )
                        ),
                    );
                }

                _ => {}
            },

            _ => {}
        };

        diag
    }
}
