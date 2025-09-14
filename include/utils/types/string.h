#ifndef __UTILS_TYPES_STRING_H__
#define __UTILS_TYPES_STRING_H__

#include <stdbool.h>
#include <stddef.h>

#include "utils/types/vector.h"

typedef struct string
{
    char *data;
    size_t cap, len;
} string_t;

// Constructors
void string_init(string_t *str, size_t cap);
string_t string_from(const char *s_lit);
string_t string_from_prt(const char *lit, size_t n);
string_t string_with(char *lit);
string_t string_with_char(char c, size_t n);

string_t string_copy(string_t *str);
void string_copy_to(string_t *str, string_t *dest);
void string_move(string_t *dest, string_t *src);

// Destructor
void string_free(string_t *str);

// Helpers

/**
 * Resets the given string and returns its allocated buffer.
 */
char *string_reset(string_t *str);

void string_setcap(string_t *str, size_t cap);

char *string_at(string_t *str, size_t idx);
char *string_silent_at(string_t *str, size_t idx);

void string_assign(string_t *str, const char *s_lit);
void string_assign_str(string_t *str, string_t *s, bool reset);

void string_push(string_t *str, char c);
void string_push_str(string_t *str, string_t *s);

size_t string_find(string_t *str, const char *needle);
size_t string_find_last(string_t *str, const char *needle);

bool string_ends_with(string_t *str, const char *suffix);

string_t string_substr(string_t *str, size_t pos, size_t n);
void string_insert(string_t *str, const char *lit, size_t pos);

void string_format(string_t *str, const char *format, ...);

T_VEC_CONSTR(string_t, str_vec,
             ((vec_opts_t){(vec_el_destr)string_free, (vec_el_cp)string_copy_to,
                           (vec_el_mv)string_move}))
T_VEC_RESET(string_t, str_vec_reset)
T_VEC_AT(string_t, str_vec)
T_VEC_INSERT(string_t, str_vec)
T_VEC_PUSH(string_t, str_vec)
T_VEC_USE(string_t, str_vec_use)

#endif
