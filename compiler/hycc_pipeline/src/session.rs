use hycc_diagnostic::DiagnosticContext;
use hycc_source::{Source, SourceRegistry};

#[derive(Debug)]
pub struct Session {
    pub source_registry: SourceRegistry,
    pub dctx: DiagnosticContext,
}

impl Session {
    pub fn new(root: Source) -> Self {
        Self {
            source_registry: SourceRegistry::new(root),
            dctx: DiagnosticContext::default(),
        }
    }
}
