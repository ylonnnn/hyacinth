use hycc_diagnostic::DiagnosticCtx;
use hycc_source::{Source, SourceRegistry};
use hycc_symbol::SymbolInterner;

#[derive(Debug)]
pub struct Session {
    pub source_registry: SourceRegistry,
    pub dctx: DiagnosticCtx,
    pub interner: SymbolInterner,
}

impl Session {
    pub fn new(root: Source) -> Self {
        Self {
            source_registry: SourceRegistry::new(root),
            dctx: DiagnosticCtx::default(),
            interner: SymbolInterner::new(),
        }
    }
}
