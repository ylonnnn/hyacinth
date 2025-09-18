#ifndef __LEXER_LEXER_H__
#define __LEXER_LEXER_H__

#include <stdint.h>
#include <string.h>

#include "core/program/program.h"
#include "core/result/result.h"
#include "lexer/token.h"
#include "utils/types/hashmap.h"
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

    hashmap_t reserved;
} lexer_t;

static inline size_t cstr_hash(void *key);
static inline bool cstr_eq(void *a, void *b);
static inline void cstr_destr(void *self);
static inline void cstr_cp(void *self, void *dest);
static inline void *cstr_alloc(void *src, size_t size);

T_HASHMAP(const char *, token_type_t, reserved,
          ((hashmap_opts_t){.hash = cstr_hash,
                            .eq = cstr_eq,
                            .key = (hashmap_data_opts_t){sizeof(const char *),
                                                         cstr_destr, cstr_cp,
                                                         NULL, cstr_alloc},
                            .val = (hashmap_data_opts_t){sizeof(token_type_t),
                                                         NULL, NULL, NULL}}))

static inline size_t cstr_hash(void *key)
{
    const char *str = *(char **)key;
    size_t hash = 0, len = strlen(str);

    for (size_t i = 0; i < len; i++)
        hash += str[i];

    return hash;
}

static inline bool cstr_eq(void *a, void *b)
{
    char *str = *(char **)a;
    return strncmp(str, *(char **)b, strlen(str)) == 0;
}

static inline void cstr_destr(void *self) { free(*(char **)self); }

static inline void cstr_cp(void *self, void *dest)
{
    char *str = *(char **)self;
    strcpy(*(char **)dest, str);
}

static inline void *cstr_alloc(void *src, [[maybe_unused]] size_t size)
{
    char **container = malloc(sizeof(src));
    *container = malloc((strlen(*(char **)src) + 1) * sizeof(char));

    return container;
}

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
void lexer_constr_token(lexer_t *lexer, size_t start, size_t end,
                        token_type_t type);

void lexer_ignore_whitespaces(lexer_t *lexer);

void lexer_read_digseq(lexer_t *lexer, uint32_t base);
void lexer_read_num(lexer_t *lexer);

size_t lexer_read_charseq(lexer_t *lexer, char terminator);
void lexer_read_char(lexer_t *lexer);
void lexer_read_str(lexer_t *lexer);

void lexer_read_ident(lexer_t *lexer);

result_t *lexer_tokenize(lexer_t *lexer);

void lexer_reserve_keywords(lexer_t *lexer);

#endif
