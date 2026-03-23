use hycc_diagnostic::DiagnosticCtx;
use hycc_source::{Source, SourceRegistry};

#[derive(Debug)]
pub struct Session {
    pub source_registry: SourceRegistry,
    pub dctx: DiagnosticCtx,
}

impl Session {
    pub fn new(root: Source) -> Self {
        Self {
            source_registry: SourceRegistry::new(root),
            dctx: DiagnosticCtx::default(),
        }
    }
}
