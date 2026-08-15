use hycc_span::Span;
use hycc_util::ternary;

#[derive(Debug, Clone)]
pub enum DiagKind {
    Info,
    Warning,
    Error(u16),
}

impl DiagKind {
    pub fn data(&self) -> (&'static str, Option<u16>) {
        let kind = match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error(..) => "error",
        };

        match self {
            Self::Info | Self::Warning => (kind, None),
            Self::Error(code) => (kind, Some(*code)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum DiagDetailKind {
    Note,
    Help,
}

#[derive(Debug, Clone)]
pub struct DiagDetail {
    pub message: String,
    pub span: Span,
    pub kind: DiagDetailKind,
}

impl DiagDetail {
    pub fn new(kind: DiagDetailKind, span: Span, message: String) -> Self {
        Self {
            span,
            message,
            kind,
        }
    }

    pub fn note(span: Span, message: String) -> Self {
        Self::new(DiagDetailKind::Note, span, message)
    }

    pub fn help(span: Span, message: String) -> Self {
        Self::new(DiagDetailKind::Help, span, message)
    }
}

#[derive(Debug, Clone)]
pub struct Diag {
    pub sub_diagnostics: Vec<Diag>,
    pub message: String,
    pub details: Vec<DiagDetail>,
    pub span: Span,
    pub kind: DiagKind,
}

impl Diag {
    pub fn new(kind: DiagKind, span: Span, message: String) -> Self {
        Self {
            sub_diagnostics: Vec::new(),
            message,
            details: Vec::new(),
            span,
            kind,
        }
    }

    pub fn info(span: Span, message: String) -> Self {
        Self::new(DiagKind::Info, span, message)
    }

    pub fn warning(span: Span, message: String) -> Self {
        Self::new(DiagKind::Warning, span, message)
    }

    pub fn error(code: u16, span: Span, message: String) -> Self {
        Self::new(DiagKind::Error(code), span, message)
    }

    pub fn add_sub_diagnostic(&mut self, sub_diagnostic: Diag) -> &mut Self {
        if sub_diagnostic.span.src_id.is_valid() {
            self.sub_diagnostics.push(sub_diagnostic);
        }

        self
    }

    pub fn note(&mut self, span: Span, message: String) -> &mut Self {
        if span.src_id.is_valid() {
            self.details.push(DiagDetail::note(span, message))
        }

        self
    }

    pub fn help(&mut self, span: Span, message: String) -> &mut Self {
        if span.src_id.is_valid() {
            self.details.push(DiagDetail::help(span, message))
        }

        self
    }
}

pub trait DiagLike {
    fn is_info(&self) -> bool;
    fn is_warning(&self) -> bool;
    fn is_error(&self) -> bool;
}

pub trait DiagEmitter<Ctx> {
    fn emit(&self, ctx: &mut Ctx) -> Diag;
}

mod sealed {
    pub trait ResultTarget<T, E> {}
    impl<T, E> ResultTarget<T, E> for Result<T, E> {}
}

pub trait FromResultEmitter<DCtx: Diagnostics<Ctx, T>, Ctx, T: DiagLike + DiagEmitter<Ctx>, RT>:
    sealed::ResultTarget<RT, T>
where
    Self: Sized,
{
    fn emit(self, dctx: &mut DCtx) -> Option<RT>;
    fn emit_discard(self, dctx: &mut DCtx) {
        self.emit(dctx);
    }
}

#[derive(Debug, Default)]
pub struct DiagCtx(Vec<Diag>, bool);

impl DiagCtx {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn data(&self) -> &[Diag] {
        &self.0
    }

    pub fn error_occurred(&self) -> Result<(), ()> {
        ternary!(self.1, Err(()), Ok(()))
    }

    pub fn add(&mut self, diag: Diag) {
        if !diag.span.src_id.is_valid() {
            return;
        }

        self.1 = self.1 || matches!(&diag.kind, DiagKind::Error(_));
        self.0.push(diag);
    }
}

pub trait Diagnostics<Ctx, T: DiagLike + DiagEmitter<Ctx>> {
    const ERROR_CODE_OFFSET: u16;

    fn data(&self) -> &[T];
    fn data_mut(&mut self) -> &mut Vec<T>;

    fn error_flag(&mut self) -> &mut bool;
    fn add(&mut self, diag: T) {
        *self.error_flag() = diag.is_error();
        self.data_mut().push(diag);
    }

    fn emit(&mut self, ctx: Ctx);
}
