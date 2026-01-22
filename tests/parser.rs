#[cfg(test)]
mod tests {
    use hyacinth::prelude::*;

    #[test]
    fn parser_variable_decl() {
        let source = "hyc/parser/var_decl.hyc";
        let mut registry = ProgramRegistry::new(Program::new(source));
        let entry = registry.entry();

        entry.compile();
    }
}
