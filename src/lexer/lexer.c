#include <ctype.h>
#include <stdint.h>

#include "core/diagnostic/code/error.h"
#include "core/diagnostic/diagnostic.h"
#include "core/result/result.h"
#include "lexer/lexer.h"
#include "lexer/token.h"
#include "lexer/token_type.h"
#include "utils/macros.h"
#include "utils/numerics.h"
#include "utils/types/string_view.h"

#define SEP(c) c == '_'

lexer_t lexer_from(program_t *program)
{
    return (lexer_t){
        .program = program,
        .tokens = token_vec_with_cap(64),
        .position = 0,
        .offset = 0,
        .row = 1,
        .col = 1,
        .p_row = 1,
        .p_col = 1,
        .result = result_create(RES_SUCCESS),
    };
}

void lexer_free(lexer_t *lexer)
{
    vec_free(&lexer->tokens);
    result_free(&lexer->result);
}

bool lexer_cbsof(lexer_t *lexer) { return lexer->offset == 0; }

bool lexer_ceof(lexer_t *lexer)
{
    return lexer->offset >= lexer->program->source.len;
}

char lexer_ccurr(lexer_t *lexer)
{
    return lexer_cbsof(lexer) || lexer_ceof(lexer)
               ? '\0'
               : *string_at(&lexer->program->source, lexer->offset - 1);
}

char lexer_cnext(lexer_t *lexer)
{
    if (lexer_ceof(lexer))
        return '\0';

    if ((lexer->offset + 1) <= lexer->program->source.len - 1)
        lexer->p_col = lexer->col++;

    return *string_at(&lexer->program->source, lexer->offset++);
}

char lexer_cpeek(lexer_t *lexer) { return lexer_cpeekn(lexer, 0); }

char lexer_cpeekn(lexer_t *lexer, size_t offset)
{
    string_t *source = &lexer->program->source;
    size_t n_pos = lexer->offset + offset;

    return n_pos >= source->len ? '\0' : *string_at(source, n_pos);
}

void lexer_cconsume(lexer_t *lexer)
{
    lexer->offset++;
    lexer->p_col = lexer->col++;
}

bool lexer_cmatch(lexer_t *lexer, char expected)
{
    return (lexer_cpeek(lexer) == expected)
               //
               ? (lexer_cconsume(lexer), true)
               : false;
}

token_t *lexer_create_token(lexer_t *lexer, size_t start, size_t end,
                            token_type_t type)
{
    program_t *program = lexer->program;
    size_t len = (end - start) + 1;

    token_t *addr = token_vec_use(&lexer->tokens);
    *addr = (token_t){
        .type = type,
        .value = sv_from(&program->source, start, len),
        .pos = (position_t){program, lexer->offset, lexer->row, lexer->col}};

    return addr;
}

void lexer_constr_token(lexer_t *lexer, size_t start, size_t end,
                        token_type_t type)
{
    lexer_create_token(lexer, start, end, type);
}

void lexer_ignore_whitespaces(lexer_t *lexer)
{
    for (char c = lexer_cpeek(lexer); !lexer_ceof(lexer) && isspace(c);
         c = lexer_cpeek(lexer))
    {
        lexer_cnext(lexer);
        if (c == '\n')
        {
            lexer->p_row = lexer->row++;
            lexer->p_col = lexer->col;
            lexer->col = 1;
        }
    }
}

void lexer_read_digseq(lexer_t *lexer, uint32_t base)
{
    bool allow_sep = false;

#define proceed(lexer)                                                         \
    lexer_cnext(lexer);                                                        \
    continue

    for (char c = lexer_cpeek(lexer); !lexer_ceof(lexer);
         c = lexer_cpeek(lexer))
    {
#define pos(lexer)                                                             \
    (position_t) { lexer->program, lexer->offset, lexer->row, lexer->col }

        if (isalnum(c))
        {
            if (!is_digit_of(c, base))
            {
                diagnostic_t diagnostic = diagnostic_from(
                    DIAG_ERROR, INVALID_NUMLIT_DIGIT,
                    string_from("invalid numeric literal digit."),
                    (position_range_t){pos(lexer), pos(lexer)});

                result_error(&lexer->result, &diagnostic);
            }

            allow_sep = true;
            proceed(lexer);
        }

        else if (SEP(c))
        {
            // If the next character is not a digit, a separator is not allowed
            // as it is the end of the digit sequence
            char n = lexer_cpeekn(lexer, 1);
            if (n == '\0' || !isdigit(n))
                allow_sep = false;

            if (!allow_sep)
            {
                diagnostic_t diagnostic = diagnostic_from(
                    DIAG_ERROR, UNEXPECTED_SEP,
                    string_from("unexpected numeric literal separator."),
                    (position_range_t){pos(lexer), pos(lexer)});

                result_error(&lexer->result, &diagnostic);
            }

            allow_sep = false;
            proceed(lexer);
        }

        return;
    }

#undef pos
#undef proceed
}

void lexer_read_num(lexer_t *lexer)
{
    //
    char c = lexer_cpeek(lexer);
    uint32_t base = 10, s_row = lexer->row, s_col = lexer->col;
    size_t s_offset = lexer->offset;

    // Prefix-ed (0b, 0o, 0x)
    if (c == '0')
    {
        lexer_cconsume(lexer), c = lexer_cpeek(lexer);
        base = c == 'b' ? 2 : c == 'o' ? 8 : c == 'x' ? 16 : UINT32_MAX;

        lexer_cconsume(lexer);
        if (lexer_ceof(lexer))
        {
            diagnostic_t diagnostic = diagnostic_from(
                DIAG_ERROR, MALFORMED_LITERAL,
                string_from("malformed numeric literal."),
                POSRNG_SWTOCURR((*lexer), s_offset, s_row, s_col));

            result_error(&lexer->result, &diagnostic);

            return;
        }
    }

    if (base == UINT32_MAX)
    {
        diagnostic_t diagnostic =
            diagnostic_from(DIAG_ERROR, INVALID_NUMLIT_PREF,
                            string_from("invalid numeric literal prefix"),
                            POSRNG_SWTOCURR((*lexer), s_offset, s_row, s_col));

        result_error(&lexer->result, &diagnostic);

        return;
    }

    lexer_read_digseq(lexer, base);

    if (lexer_cmatch(lexer, '.'))
    {
        lexer_read_digseq(lexer, base);

        lexer_create_token(lexer, s_offset, lexer->offset - 1, FLOAT_T);
        return;
    }

    lexer_create_token(lexer, s_offset, lexer->offset - 1, INT_T);
}

void lexer_read_charseq(lexer_t *lexer, char terminator)
{
    //
    TODO("lexer_read_charseq()")
}

void lexer_read_char(lexer_t *lexer)
{
    printf("%c | %c\n", lexer_cpeek(lexer), lexer_ccurr(lexer));
}

void lexer_read_str(lexer_t *lexer){//
                                    TODO("lexer_read_str()")}

result_t *lexer_tokenize(lexer_t *lexer)
{
    program_t *program = lexer->program;
    string_t *source = &program->source;

    while (!lexer_ceof(lexer))
    {
        char c = lexer_cpeek(lexer);
        size_t ofs = lexer->offset;

        if (isspace(c))
        {
            lexer_ignore_whitespaces(lexer);
            ofs = lexer->offset, c = lexer_cpeek(lexer);
        }

        if (isdigit(c))
        {
            lexer_read_num(lexer);
            continue;
        }

        if (c == '\'' || c == '"')
        {
            c == '\'' ? lexer_read_char(lexer) : lexer_read_str(lexer);
            continue;
        }

        lexer_cnext(lexer);

        switch (c)
        {
            case '{':
                lexer_constr_token(lexer, ofs, lexer->offset - 1, LEFT_BRACE);
                continue;
            case '}':
                lexer_constr_token(lexer, ofs, lexer->offset - 1, RIGHT_BRACE);
                continue;
            case '[':
                lexer_constr_token(lexer, ofs, lexer->offset - 1, LEFT_BRACKET);
                continue;
            case ']':
                lexer_constr_token(lexer, ofs, lexer->offset - 1,
                                   RIGHT_BRACKET);
                continue;
            case '(':
                lexer_constr_token(lexer, ofs, lexer->offset - 1, LEFT_PAREN);
                continue;
            case ')':
                lexer_constr_token(lexer, ofs, lexer->offset - 1, RIGHT_PAREN);
                continue;
            case ',':
                lexer_constr_token(lexer, ofs, lexer->offset - 1, COMMA);
                continue;
            case ';':
                lexer_constr_token(lexer, ofs, lexer->offset - 1, SEMICOLON);
                continue;
            case ':':
            {
                if (lexer_cmatch(lexer, ':'))
                {
                    lexer_constr_token(lexer, ofs, lexer->offset - 1,
                                       DOUBLE_COLON);
                    continue;
                }

                lexer_constr_token(lexer, ofs, lexer->offset - 1, COLON);
                continue;
            }

            case '.':
            {
                if (lexer_cmatch(lexer, '.'))
                {
                    lexer_constr_token(lexer, ofs, lexer->offset - 1,
                                       DOUBLE_DOT);
                    continue;
                }

                lexer_constr_token(lexer, ofs, lexer->offset - 1, DOT);
                continue;
            }

            case '<':
            {
                // <-
                if (lexer_cmatch(lexer, '-'))
                {
                    char p = lexer_cpeek(lexer);
                    if (p == '\0')
                        break;

                    if (isdigit(p))
                    {
                        lexer_constr_token(lexer, ofs, lexer->offset - 1, LESS);
                        continue;
                    }

                    lexer_constr_token(lexer, ofs, lexer->offset - 1,
                                       LESS_MINUS);
                    continue;
                }

                // <=
                else if (lexer_cmatch(lexer, '='))
                {
                    lexer_constr_token(lexer, ofs, lexer->offset - 1,
                                       LESS_EQUAL);
                    continue;
                }

                // <
                lexer_constr_token(lexer, ofs, lexer->offset - 1, LESS_EQUAL);
                continue;
            }

            case '>':
            {
                // >=
                if (lexer_cmatch(lexer, '='))
                {
                    lexer_constr_token(lexer, ofs, lexer->offset - 1,
                                       GREATER_EQUAL);
                    continue;
                }

                // >
                lexer_constr_token(lexer, ofs, lexer->offset - 1, GREATER);
                continue;
            }

            case '=':
            {
                // ==
                if (lexer_cmatch(lexer, '='))
                {
                    lexer_constr_token(lexer, ofs, lexer->offset - 1,
                                       EQUAL_EQUAL);
                    continue;
                }

                // =
                lexer_constr_token(lexer, ofs, lexer->offset - 1, EQUAL);
                continue;
            }

            case '+':
            {
                // +=
                if (lexer_cmatch(lexer, '='))
                {
                    lexer_constr_token(lexer, ofs, lexer->offset - 1,
                                       PLUS_EQUAL);
                    continue;
                }

                else if (lexer_cmatch(lexer, '+'))
                    lexer_constr_token(lexer, ofs, lexer->offset - 1,
                                       PLUS_PLUS);
                continue;

                // +
                lexer_constr_token(lexer, ofs, lexer->offset - 1, PLUS);
                continue;
            }

            case '-':
            {
                char p = lexer_cpeek(lexer);
                if (p == '\0')
                    break;

                if (isdigit(p))
                {
                    lexer_read_num(lexer);
                    continue;
                }

                // -=
                else if (lexer_cmatch(lexer, '='))
                {
                    lexer_constr_token(lexer, ofs, lexer->offset - 1,
                                       MINUS_EQUAL);
                    continue;
                }

                else if (lexer_cmatch(lexer, '-'))
                {
                    lexer_constr_token(lexer, ofs, lexer->offset - 1,
                                       MINUS_MINUS);
                    continue;
                }
                else if (lexer_cmatch(lexer, '>'))
                {
                    lexer_constr_token(lexer, ofs, lexer->offset - 1,
                                       MINUS_GREATER);
                    continue;
                }

                // -
                lexer_constr_token(lexer, ofs, lexer->offset - 1, MINUS);
                continue;
            }

            case '*':
            {
                // *=
                if (lexer_cmatch(lexer, '='))
                {
                    lexer_constr_token(lexer, ofs, lexer->offset - 1,
                                       STAR_EQUAL);
                    continue;
                }

                // *
                lexer_constr_token(lexer, ofs, lexer->offset - 1, STAR);
                continue;
            }

            case '/':
            {
                // //
                if (lexer_cmatch(lexer, '/'))
                {
                    while (!lexer_cmatch(lexer, '\n'))
                        continue;

                    lexer->p_row = lexer->row++;
                    lexer->p_col = lexer->col;
                    lexer->col = 1;

                    continue;
                }

                // /=
                else if (lexer_cmatch(lexer, '='))
                {
                    lexer_constr_token(lexer, ofs, lexer->offset - 1,
                                       SLASH_EQUAL);
                    continue;
                }

                // /
                lexer_constr_token(lexer, ofs, lexer->offset - 1, SLASH_EQUAL);
                continue;
            }

            case '%':
            {
                // %=
                if (lexer_cmatch(lexer, '='))
                {
                    lexer_constr_token(lexer, ofs, lexer->offset - 1,
                                       PERCENT_EQUAL);
                    continue;
                }

                // %
                lexer_constr_token(lexer, ofs, lexer->offset - 1, PERCENT);
                continue;
            }

            case '!':
            {
                // !=
                if (lexer_cmatch(lexer, '='))
                {
                    lexer_constr_token(lexer, ofs, lexer->offset - 1,
                                       BANG_EQUAL);
                    continue;
                }

                // !
                lexer_constr_token(lexer, ofs, lexer->offset - 1, BANG);
                continue;
            }

            case '&':
            {
                // &&
                if (lexer_cmatch(lexer, '&'))
                {
                    lexer_constr_token(lexer, ofs, lexer->offset - 1,
                                       AMPERSAND_AMPERSAND);
                    continue;
                }

                // &
                lexer_constr_token(lexer, ofs, lexer->offset - 1, AMPERSAND);
                continue;
            }

            case '|':
            {
                // ||
                if (lexer_cmatch(lexer, '|'))
                {
                    lexer_constr_token(lexer, ofs, lexer->offset - 1,
                                       PIPE_PIPE);
                    continue;
                }

                // |
                lexer_constr_token(lexer, ofs, lexer->offset - 1, PIPE);
                continue;
            }

            case '^':
            {
                // ^^ (Exponent (?))
                if (lexer_cmatch(lexer, '^'))
                {
                    lexer_constr_token(lexer, ofs, lexer->offset - 1,
                                       CARET_CARET);
                    continue;
                }

                // ^
                lexer_constr_token(lexer, ofs, lexer->offset - 1, CARET);
                continue;
            }
            default:
            {
                if (lexer_ceof(lexer))
                    break;

                lexer_constr_token(lexer, ofs, lexer->offset - 1, ILLEGAL);
            }
        }
        break;
    }

    // EOF
    lexer_create_token(lexer, source->len, source->len, END_OF_FILE);

    return &lexer->result;
}
