use hycc_const::table::ConstTable;
use hycc_diagnostic::diagnostic::DiagCtx;
use hycc_hir::{HirTable, def::DefinitionTable, petal::PetalCtx};
use hycc_source::{Source, SourceRegistry, source::SourceId};
use hycc_symbol::SymbolInterner;
use hycc_ty::ctx::TyCtx;

#[derive(Debug)]
pub struct Session<'h> {
    pub tctx: TyCtx,
    pub definitions: DefinitionTable,
    pub petal_ctx: PetalCtx,
    pub interner: SymbolInterner,
    pub const_table: ConstTable,
    pub dctx: DiagCtx,
    pub hir_table: HirTable<'h>,
    pub root: SourceId,
}

impl<'h> Session<'h> {
    pub fn new(root: SourceId) -> Self {
        Self {
            tctx: TyCtx::new(),
            definitions: DefinitionTable::new(),
            petal_ctx: PetalCtx::new(),
            dctx: DiagCtx::default(),
            interner: SymbolInterner::new(),
            hir_table: HirTable::new(),
            const_table: ConstTable::new(),
            root,
        }
    }
}
