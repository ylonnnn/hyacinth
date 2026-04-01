use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(usize);

impl Symbol {
    #[allow(non_snake_case)]
    pub fn Invalid() -> Self {
        Self(usize::MAX)
    }

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "symbol id must be valid");
        self.0
    }
}

#[derive(Debug)]
pub struct SymbolInterner {
    data: Vec<Box<str>>,
    symbols: HashMap<Box<str>, Symbol>,
}

impl SymbolInterner {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            symbols: HashMap::new(),
        }
    }

    pub fn intern(&mut self, data: &str) -> Symbol {
        use std::collections::hash_map::Entry;

        match self.symbols.entry(data.into()) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let sym = Symbol(self.data.len());

                self.data.push(data.into());
                entry.insert(sym);

                sym
            }
        }
    }

    pub fn get(&self, id: Symbol) -> &str {
        &self.data[id.unwrap()]
    }
}
