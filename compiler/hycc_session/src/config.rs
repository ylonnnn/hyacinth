use hycc_ast::token::TokenKind;

#[macro_export]
macro_rules! file_ext {
    () => {
        "hyc"
    };
}

pub const HYC_FILE_EXT: &str = file_ext!();
pub const HYC_DIR_PETAL_FILE: &str = concat!("petal", ".", file_ext!());

pub const HYC_PATH_SEP_TOK_KIND: TokenKind = TokenKind::ColonColon;
