#ifndef __LEXER_TOKEN_H__
#define __LEXER_TOKEN_H__

#include "core/position/position.h"
#include "lexer/token_type.h"
#include "utils/types/string_view.h"
#include "utils/types/vector.h"

typedef struct token
{
    token_type_t type;
    string_view_t value;
    position_t pos;
} token_t;

// Constructors
token_t token_from(token_type_t type, string_view_t value, position_t pos);
token_t *token_create_on(token_t *addr, token_type_t type, string_view_t value,
                         position_t pos);

// Helper

string_t token_to_str(token_t *token);

void token_print(token_t *token);

T_VEC(token_t, token_vec, (vec_opts_t){NULL})

#endif
