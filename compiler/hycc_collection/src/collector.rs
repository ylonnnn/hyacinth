use hycc_ast::Program;
use hycc_source::Source;
use hycc_symbol::SymbolInterner;

#[derive(Debug)]
pub struct Collector<'i, 's> {
    interner: &'i mut SymbolInterner,
    source: &'s Source,
}

impl<'i, 's> Collector<'i, 's> {
    pub fn new(interner: &'i mut SymbolInterner, source: &'s Source) -> Self {
        Self { interner, source }
    }

    pub fn collect(&mut self, tree: Program) {
        todo!()
    }
}
