use hyacinth::core::{Program, ProgramRegistry};

fn main() {
    let mut registry = ProgramRegistry::new(Program::new("hyc/lexer/first.hyc"));
    let entry = registry.entry();

    entry.compile();
}
