use hycc_symbol::SymbolInterner;

#[derive(Debug)]
pub struct SymCollector<'i> {
    interner: &'i mut SymbolInterner,
}

impl<'i> SymCollector<'i> {
    pub fn new(interner: &'i mut SymbolInterner) -> Self {
        Self { interner }
    }

    pub fn collect(&mut self) {
        todo!()
    }
}
