use hyacinth::core::{Program, ProgramRegistry};

fn main() {
    let entry = Program::new();

    #[allow(unused)]
    let registry = ProgramRegistry::new(entry);
}
