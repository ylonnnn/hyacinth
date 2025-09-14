#ifndef __LEXER_TOKEN_TYPE_H__
#define __LEXER_TOKEN_TYPE_H__

typedef enum token_type
{
    // ==== ILLEGAL ====
    ILLEGAL,

    // ==== RESERVED ====

    // ==== RESERVED::DECLARATION ====
    STRUCT,
    FNC,
    LET,

    // ==== RESERVED::QUALIFIER ====
    VAR,

    // ==== RESERVED::CONTROL ====
    RETURN,

    // ==== RESERVED::ACCESSIBILITY ====
    PUB,
    PRIV,
    PROT,

    // ==== RESERVED::OTHER ====
    USE,

    // ==== PRIMARY ====
    IDENTIFIER,
    INT_T,
    FLOAT_T,
    CHAR_T,
    BOOL_T,
    STR_T,
    NULL_T,

    // ==== OPERATORS ====

    // ==== OPERATORS:ARITHMETIC ====
    PLUS,    // +
    MINUS,   // -
    STAR,    // *
    SLASH,   // /
    PERCENT, // %

    // ==== OPERATORS::ARROW ====
    LESS_MINUS,    // <-
    MINUS_GREATER, // ->

    // ===== OPERATORS:COMPARISON =====
    EQUAL_EQUAL,   // ==
    BANG_EQUAL,    // !=
    LESS,          // <
    LESS_EQUAL,    // <=
    GREATER,       // >
    GREATER_EQUAL, // >=

    // ===== OPERATORS:LOGICAL =====
    BANG,                // !
    AMPERSAND_AMPERSAND, // &&
    PIPE_PIPE,           // ||

    // ===== OPERATORS:BITWISE =====
    AMPERSAND,       // &
    PIPE,            // |
    CARET,           // ^
    TILDE,           // ~
    LESS_LESS,       // <<
    GREATER_GREATER, // >>

    // ==== OPERATORS::UNARY ====
    PLUS_PLUS,   // ++
    MINUS_MINUS, // --
    CARET_CARET, // ^^ (Exponent)

    // ===== OPERATORS:ASSIGNMENT =====
    EQUAL,         // =
    PLUS_EQUAL,    // +=
    MINUS_EQUAL,   // -=
    STAR_EQUAL,    // *=
    SLASH_EQUAL,   // /=
    PERCENT_EQUAL, // %=

    // ===== DELIMITERS =====
    LEFT_PAREN,    // (
    RIGHT_PAREN,   // )
    LEFT_BRACE,    // {
    RIGHT_BRACE,   // {}
    LEFT_BRACKET,  // [
    RIGHT_BRACKET, // ]
    COMMA,         // ,
    DOT,           // .
    DOUBLE_DOT,    // ..
    SEMICOLON,     // ;
    COLON,         // :
    DOUBLE_COLON,  // ::
    QUESTION,      // ?

    // ==== MISC ====
    END_OF_FILE, // EOF

    TOKEN_TYPE_COUNT
} token_type_t;

static const char *TOKEN_TYPE_STRING_REPR[TOKEN_TYPE_COUNT] = {
    // ==== ILLEGAL ====
    [ILLEGAL] = "Illegal",

    // ==== RESERVED ====

    // ==== RESERVED::DECLARATION ====
    [STRUCT] = "struct",
    [FNC] = "fn",
    [LET] = "let",

    // ==== RESERVED::QUALIFIER ====
    [VAR] = "var",

    // ==== RESERVED::CONTROL ====
    [RETURN] = "return",

    // ==== RESERVED::ACCESSIBILITY ====
    [PUB] = "pub",
    [PRIV] = "priv",
    [PROT] = "prot",

    // ==== RESERVED::OTHER ====
    [USE] = "use",

    // ==== PRIMARY ====
    [IDENTIFIER] = "identifier",
    [INT_T] = "int",
    [FLOAT_T] = "float",
    [CHAR_T] = "char",
    [BOOL_T] = "bool",
    [STR_T] = "string",
    [NULL_T] = "null",

    // ==== OPERATORS ====

    // ==== OPERATORS:ARITHMETIC ====
    [PLUS] = "+",
    [MINUS] = "-",
    [STAR] = "*",
    [SLASH] = "/",
    [PERCENT] = "%",

    // ==== OPERATORS::ARROW ====
    [LESS_MINUS] = "<-",
    [MINUS_GREATER] = "->",

    // ===== OPERATORS:COMPARISON =====
    [EQUAL_EQUAL] = "==",
    [BANG_EQUAL] = "!=",
    [LESS] = "<",
    [LESS_EQUAL] = "<=",
    [GREATER] = ">",
    [GREATER_EQUAL] = ">=",

    // ===== OPERATORS:LOGICAL =====
    [BANG] = "!",
    [AMPERSAND_AMPERSAND] = "&&",
    [PIPE_PIPE] = "||",

    // ===== OPERATORS:BITWISE =====
    [AMPERSAND] = "&",
    [PIPE] = "|",
    [CARET] = "^",
    [TILDE] = "~",
    [LESS_LESS] = "<<",
    [GREATER_GREATER] = ">>",

    // ==== OPERATORS::UNARY ====
    [PLUS_PLUS] = "++",
    [MINUS_MINUS] = "--",
    [CARET_CARET] = "^^",

    // ===== OPERATORS:ASSIGNMENT =====
    [EQUAL] = "=",
    [PLUS_EQUAL] = "+=",
    [MINUS_EQUAL] = "-=",
    [STAR_EQUAL] = "*=",
    [SLASH_EQUAL] = "/=",
    [PERCENT_EQUAL] = "%=",

    // ===== DELIMITERS =====
    [LEFT_PAREN] = "(",
    [RIGHT_PAREN] = ")",
    [LEFT_BRACE] = "{",
    [RIGHT_BRACE] = "}",
    [LEFT_BRACKET] = "[",
    [RIGHT_BRACKET] = "]",
    [COMMA] = ",",
    [DOT] = ".",
    [DOUBLE_DOT] = "..",
    [SEMICOLON] = ";",
    [COLON] = ":",
    [DOUBLE_COLON] = "::",
    [QUESTION] = "?",

    // ==== MISC ====
    [END_OF_FILE] = "EOF"};

static const char *TOKEN_TYPE_NAMES[TOKEN_TYPE_COUNT] = {
    // ==== ILLEGAL ====
    [ILLEGAL] = "Illegal",

    // ==== RESERVED ====

    // ==== RESERVED::DECLARATION ====
    [STRUCT] = "Struct",
    [FNC] = "Fn",
    [LET] = "Let",

    // ==== RESERVED::QUALIFIER ====
    [VAR] = "Var",

    // ==== RESERVED::CONTROL ====
    [RETURN] = "Return",

    // ==== RESERVED::ACCESSIBILITY ====
    [PUB] = "Pub",
    [PRIV] = "Priv",
    [PROT] = "Prot",

    // ==== RESERVED::OTHER ====
    [USE] = "Use",

    // ==== PRIMARY ====
    [IDENTIFIER] = "Identifier",
    [INT_T] = "Int",
    [FLOAT_T] = "Float",
    [CHAR_T] = "Char",
    [BOOL_T] = "Bool",
    [STR_T] = "String",
    [NULL_T] = "Null",

    // ==== OPERATORS ====

    // ==== OPERATORS:ARITHMETIC ====
    [PLUS] = "Plus",
    [MINUS] = "Minus",
    [STAR] = "Star",
    [SLASH] = "Slash",
    [PERCENT] = "Percent",

    // ==== OPERATORS::ARROW ====
    [LESS_MINUS] = "LessMinus",
    [MINUS_GREATER] = "MinusGreater",

    // ===== OPERATORS:COMPARISON =====
    [EQUAL_EQUAL] = "EqualEqual",
    [BANG_EQUAL] = "BangEqual",
    [LESS] = "Less",
    [LESS_EQUAL] = "LessEqual",
    [GREATER] = "Greater",
    [GREATER_EQUAL] = "GreaterEqual",

    // ===== OPERATORS:LOGICAL =====
    [BANG] = "Bang",
    [AMPERSAND_AMPERSAND] = "AmpersandAmpersand",
    [PIPE_PIPE] = "PipePipe",

    // ===== OPERATORS:BITWISE =====
    [AMPERSAND] = "Ampersand",
    [PIPE] = "Pipe",
    [CARET] = "Caret",
    [TILDE] = "Tilde",
    [LESS_LESS] = "LessLess",
    [GREATER_GREATER] = "GreaterGreater",

    // ==== OPERATORS::UNARY ====
    [PLUS_PLUS] = "PlusPlus",
    [MINUS_MINUS] = "MinusMinus",
    [CARET_CARET] = "CaretCaret",

    // ===== OPERATORS:ASSIGNMENT =====
    [EQUAL] = "Equal",
    [PLUS_EQUAL] = "PlusEqual",
    [MINUS_EQUAL] = "MinusEqual",
    [STAR_EQUAL] = "StarEqual",
    [SLASH_EQUAL] = "SlashEqual",
    [PERCENT_EQUAL] = "PercentEqual",

    // ===== DELIMITERS =====
    [LEFT_PAREN] = "LeftParen",
    [RIGHT_PAREN] = "RightParen",
    [LEFT_BRACE] = "LeftBrace",
    [RIGHT_BRACE] = "RightBrace",
    [LEFT_BRACKET] = "LeftBracket",
    [RIGHT_BRACKET] = "RightBracket",
    [COMMA] = "Comma",
    [DOT] = "Dot",
    [DOUBLE_DOT] = "DoubleDot",
    [SEMICOLON] = "Semicolon",
    [COLON] = "Colon",
    [DOUBLE_COLON] = "DoubleColon",
    [QUESTION] = "Question",

    // ==== MISC ====
    [END_OF_FILE] = "EOF"};

static inline const char *token_type_to_string(token_type_t type)
{
    return TOKEN_TYPE_STRING_REPR[type];
}

static inline const char *token_type_to_name(token_type_t type)
{
    return TOKEN_TYPE_NAMES[type];
}

#endif
