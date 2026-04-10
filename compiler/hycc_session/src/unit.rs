use hycc_hir::HirTable;
use hycc_source::source::SourceId;

#[derive(Debug)]
pub struct CompilationUnit<'h> {
    pub hir_table: HirTable<'h>,
    pub root: SourceId,
}

impl<'h> CompilationUnit<'h> {
    pub fn new(root: SourceId) -> Self {
        Self {
            hir_table: HirTable::new(),
            root,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompilationUnitId(pub(crate) usize);

impl CompilationUnitId {
    #[allow(non_upper_case_globals)]
    pub const Invalid: Self = Self(usize::MAX);

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "compilation unit id is not valid!");
        self.0
    }
}
