use hycc_diagnostic::DiagnosticCtx;
use hycc_source::{Source, SourceRegistry};
use hycc_symbol::SymbolInterner;

use crate::unit::{CompilationUnit, CompilationUnitId};

#[derive(Debug)]
pub struct Session<'h> {
    pub interner: SymbolInterner,
    pub dctx: DiagnosticCtx,
    pub registry: SourceRegistry,
    pub units: Vec<CompilationUnit<'h>>,
}

impl<'h> Session<'h> {
    pub fn new() -> Self {
        Self {
            dctx: DiagnosticCtx::default(),
            interner: SymbolInterner::new(),
            registry: SourceRegistry::new(),
            units: Vec::new(),
        }
    }

    pub fn create_unit(&mut self, root: Source) -> CompilationUnitId {
        let root_id = self.registry.register(root);

        self.units.push(CompilationUnit::new(root_id));
        CompilationUnitId(self.units.len() - 1)
    }

    pub fn get_unit(&self, id: CompilationUnitId) -> &CompilationUnit<'h> {
        &self.units[id.unwrap()]
    }
}
