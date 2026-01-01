#[cfg(test)]
mod tests {
    use LexerSourceOrigin::*;
    use hyacinth::prelude::*;

    #[test]
    fn lexer_basic_binary() {
        let source = r"1 + 2";
        let mut lexer = Lexer::new(Arbitrary(source.to_owned()));

        lexer.tokenize();
    }

    #[test]
    fn lexer_from_file() {
        let source = "hyc/lexer/first.hyc";
        let mut lexer = Lexer::new(File(source.to_owned()));

        lexer.tokenize();
    }
}
