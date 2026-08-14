use hycc_const::table::ConstTable;
use hycc_diagnostic::diagnostic::DiagCtx;
use hycc_hir::HirTable;
use hycc_source::{Source, SourceRegistry, source::SourceId};
use hycc_symbol::SymbolInterner;

#[derive(Debug)]
pub struct Session<'h> {
    pub interner: SymbolInterner,
    pub const_table: ConstTable,
    pub dctx: DiagCtx,
    pub hir_table: HirTable<'h>,
    pub root: SourceId,
}

impl<'h> Session<'h> {
    pub fn new(root: SourceId) -> Self {
        Self {
            dctx: DiagCtx::default(),
            interner: SymbolInterner::new(),
            hir_table: HirTable::new(),
            const_table: ConstTable::new(),
            root,
        }
    }
}
