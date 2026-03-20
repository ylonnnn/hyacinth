use hycc_parser::Lexer;
use hycc_source::Source;

pub fn compile(source: &Source) {
    let mut lexer = Lexer::new(source);
    lexer.tokenize();
}

pub fn compile_arbitrary_str(source: String) {
    compile(&Source::new(
        String::from("hyacinth.hyc"),
        String::from(source),
    ));
}

pub fn compile_file(path: &str) {
    compile(&Source::new_from_file(path));
}
