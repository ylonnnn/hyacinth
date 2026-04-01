use hycc_span::Span;

#[derive(Debug)]
pub struct DefinitionTable {
    data: Vec<Definition>,
}

impl DefinitionTable {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn insert(&mut self, definition: Definition) -> DefId {
        self.data.push(definition);
        DefId(self.data.len() - 1)
    }

    pub fn get(&self, id: DefId) -> &Definition {
        &self.data[id.unwrap()]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DefId(usize);

impl DefId {
    #[allow(non_snake_case)]
    pub fn Invalid() -> Self {
        Self(usize::MAX)
    }

    pub fn unwrap(&self) -> usize {
        assert_ne!(self.0, usize::MAX, "definition id is not valid!");
        self.0
    }
}

#[derive(Debug, Clone)]
pub enum DefKind {
    Fn,
}

impl DefKind {
    pub fn space(&self) -> DefSpace {
        match self {
            Self::Fn => DefSpace::Value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DefSpace {
    Type,
    Value,
}

#[derive(Debug, Clone)]
pub struct Definition {
    // pub kind
    pub span: Span,
    // TODO: visibility
}
