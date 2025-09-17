#include <assert.h>

#include "lexer/token.h"
#include "lexer/token_type.h"
#include "utils/macros.h"

token_t token_from(token_type_t type, string_view_t value, position_t pos)
{
    return (token_t){
        .type = type,
        .value = value,
        .pos = pos,
    };
}

token_t *token_create_on(token_t *addr, token_type_t type, string_view_t value,
                         position_t pos)
{
    assert(addr != NULL);

    *addr = (token_t){
        .type = type,
        .value = value,
        .pos = pos,
    };

    return addr;
}

string_t token_to_str(token_t *token)
{
    position_t *pos = &token->pos;
    string_view_t *value = &token->value;

    string_t str;

    string_init(&str, 256);
    string_format(&str, "<%s:%u:%u> ", token_type_to_name(token->type),
                  pos->row, pos->col);

    clean(string_free) string_t t_val =
        string_from_prt(value->data, value->len);
    string_push_str(&str, &t_val);

    return str;
}

void token_print(token_t *token)
{
    clean(string_free) string_t str = token_to_str(token);
    printf("%s", str.data);
}
