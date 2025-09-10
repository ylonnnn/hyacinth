#ifndef __LEXER_LEXER_H__
#define __LEXER_LEXER_H__

#include <stdint.h>

#include "core/program/program.h"
#include "core/result/result.h"
#include "lexer/token.h"
#include "utils/types/vector.h"

#define POSRNG_SWTOCURR(lexer, s_offset, s_row, s_col)                         \
    (position_range_t)                                                         \
    {                                                                          \
        (position_t){lexer.program, s_offset, s_row, s_col}, (position_t)      \
        {                                                                      \
            lexer.program, lexer.offset, lexer.p_row, lexer.p_col              \
        }                                                                      \
    }

typedef struct lexer
{
    program_t *program;

    size_t position, offset;
    uint32_t row, col, p_row, p_col;

    vector_t tokens;
    result_t result;
} lexer_t;

// Constructors
lexer_t lexer_from(program_t *program);

// Destructors
void lexer_free(lexer_t *lexer);

// Helper

// bool lexer_bsof(lexer_t *lexer);
// bool lexer_eof(lexer_t *lexer);

bool lexer_cbsof(lexer_t *lexer);
bool lexer_ceof(lexer_t *lexer);

char lexer_ccurr(lexer_t *lexer);
char lexer_cnext(lexer_t *lexer);
char lexer_cpeek(lexer_t *lexer);
char lexer_cpeekn(lexer_t *lexer, size_t offset);

void lexer_cconsume(lexer_t *lexer);
bool lexer_cmatch(lexer_t *lexer, char expected);

token_t *lexer_create_token(lexer_t *lexer, size_t start, size_t end,
                            token_type_t type);

void lexer_ignore_whitespaces(lexer_t *lexer);

void lexer_read_digseq(lexer_t *lexer, uint32_t base);
void lexer_read_num(lexer_t *lexer);

void lexer_tokenize(lexer_t *lexer);

#endif
