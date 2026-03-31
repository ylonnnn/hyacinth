use hycc_symbol::Symbol;

#[derive(Debug)]
pub enum IdentityKind {
    // Type Namespace
    Petal,

    // Value Namespace
    Var(/* TODO: Ty */),
}

#[derive(Debug)]
pub struct Identity {
    pub name: Symbol,
    pub kind: IdentityKind,
}

impl Identity {
    pub fn new(name: Symbol, kind: IdentityKind) -> Self {
        Self { name, kind }
    }
}
