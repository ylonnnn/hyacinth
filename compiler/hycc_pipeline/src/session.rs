use hycc_diagnostic::DiagnosticCtx;
use hycc_hir::HirTable;
use hycc_source::{Source, SourceRegistry};
use hycc_symbol::SymbolInterner;

#[derive(Debug)]
pub struct Session<'h> {
    pub source_registry: SourceRegistry,
    pub dctx: DiagnosticCtx,
    pub interner: SymbolInterner,
    pub hir_table: HirTable<'h>,
}

impl<'h> Session<'h> {
    pub fn new(root: Source) -> Self {
        Self {
            source_registry: SourceRegistry::new(root),
            dctx: DiagnosticCtx::default(),
            interner: SymbolInterner::new(),
            hir_table: HirTable::new(),
        }
    }
}
