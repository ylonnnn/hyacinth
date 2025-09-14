#ifndef __UTILS_TYPES_STRING_VIEW_H__
#define __UTILS_TYPES_STRING_VIEW_H__
#include <stddef.h>
#include <stdio.h>

#include "utils/types/string.h"
#include "utils/types/vector.h"

typedef struct string_view
{
    const char *data;
    size_t len;
} string_view_t;

string_view_t sv_from(string_t *str, size_t start, size_t n);
string_view_t sv_from_sv(string_view_t *str, size_t start, size_t n);

string_view_t sv_copy(string_view_t *sv);
void sv_copy_to(string_view_t *sv, string_view_t *dest);

string_view_t sv_substr(string_view_t *sv, size_t pos, size_t n);

void sv_print(string_view_t *sv, FILE *stream);

T_VEC_CONSTR(string_view_t, sv_vec,
             ((vec_opts_t){NULL, (vec_el_cp)sv_copy_to, NULL}))
T_VEC_RESET(string_view_t, sv_vec_reset)
T_VEC_AT(string_view_t, sv_vec)
T_VEC_INSERT(string_view_t, sv_vec)
T_VEC_PUSH(string_view_t, sv_vec)
T_VEC_USE(string_view_t, sv_vec_use)

#endif
