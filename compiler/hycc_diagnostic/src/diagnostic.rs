use hycc_span::Span;

#[derive(Debug, Clone)]
pub enum DiagnosticKind {
    Note(u16, String),
    Warning(u16, String),
    Error(u16, String),
}

impl DiagnosticKind {
    pub fn data(&self) -> (&'static str, u16, &String) {
        let kind = match self {
            Self::Note(..) => "note",
            Self::Warning(..) => "warning",
            Self::Error(..) => "error",
        };

        match self {
            Self::Note(code, message)
            | Self::Warning(code, message)
            | Self::Error(code, message) => (kind, *code, message),
        }
    }
}

pub trait IntoDiagnostic {
    fn into_diag(&self, kind: DiagnosticKind) -> Diagnostic;
}

impl IntoDiagnostic for Span {
    fn into_diag(&self, kind: DiagnosticKind) -> Diagnostic {
        Diagnostic::new(*self, kind)
    }
}

pub trait Diag<Ctx> {
    fn emit(&self, ctx: &Ctx) -> Diagnostic;
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub details: Vec<Diagnostic>,
    pub span: Span,
    pub kind: DiagnosticKind,
}

impl Diagnostic {
    pub fn new(span: Span, kind: DiagnosticKind) -> Self {
        Self {
            span,
            kind,
            details: Vec::new(),
        }
    }

    pub fn is_note(&self) -> bool {
        matches!(self.kind, DiagnosticKind::Note(..))
    }

    pub fn is_warning(&self) -> bool {
        matches!(self.kind, DiagnosticKind::Warning(..))
    }

    pub fn is_error(&self) -> bool {
        matches!(self.kind, DiagnosticKind::Error(..))
    }

    pub fn add_detail(&mut self, detail: Diagnostic) -> &mut Self {
        self.details.push(detail);
        self
    }

    pub fn detail(&mut self, span: Span, kind: DiagnosticKind) -> &mut Self {
        self.add_detail(Diagnostic::new(span, kind));
        self
    }
}

pub trait DiagnosticContext<Ctx, T = Diagnostic> {
    fn data(&self) -> &Vec<T>;
    fn data_mut(&mut self) -> &mut Vec<T>;

    fn error_occurred(&self) -> bool;

    fn add(&mut self, diagnostic: T) -> Option<&mut T> {
        let data = self.data_mut();

        data.push(diagnostic);
        data.last_mut()
    }

    fn emit(&self, target: &mut DiagnosticCtx, ctx: Ctx);
}

#[derive(Debug)]
pub struct DiagnosticCtx(Vec<Diagnostic>, bool);

impl Default for DiagnosticCtx {
    fn default() -> Self {
        Self(Vec::with_capacity(32), false)
    }
}

impl DiagnosticCtx {
    pub fn new() -> Self {
        Self(Vec::new(), false)
    }
}

impl DiagnosticContext<()> for DiagnosticCtx {
    fn data(&self) -> &Vec<Diagnostic> {
        &self.0
    }

    fn data_mut(&mut self) -> &mut Vec<Diagnostic> {
        &mut self.0
    }

    fn add(&mut self, diagnostic: Diagnostic) -> Option<&mut Diagnostic> {
        self.1 = self.1 && diagnostic.is_error();
        let data = self.data_mut();

        data.push(diagnostic);
        data.last_mut()
    }

    fn error_occurred(&self) -> bool {
        self.1
    }

    fn emit(&self, _target: &mut DiagnosticCtx, _ctx: ()) {}
}
