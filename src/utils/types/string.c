#include <assert.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "utils/control.h"
#include "utils/macros.h"
#include "utils/types/string.h"

void string_init(string_t *str, size_t cap)
{
    assert(str != NULL && cap > 0);

    char *temp = malloc((cap + 1) * sizeof(char));
    if (temp == NULL)
        terminate(
            "[string] failed to allocate memory for the buffer of the string.",
            EXIT_FAILURE);

    str->data = temp;
    str->cap = cap;
    str->len = 0;
}

string_t string_from(const char *s_lit)
{
    return string_from_prt(s_lit, strlen(s_lit));
}

string_t string_from_prt(const char *lit, size_t n)
{
    string_t str = {
        .data = NULL,
        .len = n,
        .cap = n + 1,
    };

    char *buf = malloc((str.cap) * sizeof(char));
    if (buf == NULL)
        terminate(
            "[string] failed to allocate memory for the buffer of the string.",
            EXIT_FAILURE);

    memcpy(buf, lit, n + 1);
    buf[n] = '\0';

    str.data = buf;

    return str;
}

string_t string_with(char *lit)
{
    size_t len = strlen(lit);

    return (string_t){
        .data = lit,
        .len = len,
        .cap = len + 1,
    };
}

string_t string_with_char(char c, size_t n)
{
    string_t str;
    string_init(&str, n + 1);

    str.len = n;
    memset(str.data, c, n);

    str.data[n] = '\0';

    return str;
}

void string_move(string_t *dest, string_t *src)
{
    assert(dest != NULL && src != NULL);

    dest->len = src->len;
    dest->cap = src->cap;
    dest->data = string_reset(src);
}

void string_free(string_t *str)
{
    assert(str != NULL);

    free(string_reset(str));
}

char *string_reset(string_t *str)
{
    assert(str != NULL);

    char *buf = str->data;

    str->data = NULL;
    str->len = 0;
    str->cap = 0;

    return buf;
}

void string_setcap(string_t *str, size_t cap)
{
    assert(str != NULL && cap > 0);

    char *buf = realloc(str->data, cap * sizeof(char));
    if (buf == NULL)
        terminate("[string] failed to reallocate memory for the buffer of the "
                  "string.",
                  EXIT_FAILURE);

    str->data = buf;
    str->cap = cap;

    if (str->len > cap)
        str->len = cap - 1;
}

void string_assign(string_t *str, const char *s_lit)
{
    assert(str != NULL && s_lit != NULL);

    size_t n_cap = strlen(s_lit) + 1;
    string_setcap(str, n_cap);

    memmove(str->data, s_lit, n_cap);
    str->len = n_cap - 1;
}

void string_assign_str(string_t *str, string_t *s, bool reset)
{
    string_assign(str, reset ? s->data : string_reset(s));
}

void string_push(string_t *str, char c)
{
    assert(str != NULL);

    size_t n_cap = str->len + 1;

    if ((n_cap + 1) >= str->cap)
        string_setcap(str, MAX(n_cap, str->cap * 2));

    str->data[str->len] = c;
    str->len = n_cap;
}

void string_push_str(string_t *str, string_t *s)
{
    assert(str != NULL && s != NULL);

    size_t n_len = str->len + s->len;
    size_t n_cap = str->cap;

    while (n_len > n_cap)
        n_cap = n_cap == 0 ? 1 : n_cap * 2;

    if (n_cap > str->cap)
        string_setcap(str, n_cap);

    memmove(str->data + str->len, s->data, s->len);
    str->len += s->len;
}

size_t string_find(string_t *str, const char *needle)
{
    assert(str != NULL && needle != NULL);

    for (size_t i = 0; i < str->len; i++)
        if (!strncmp(str->data + i, needle, strlen(needle)))
            return i;

    return SIZE_MAX;
}

size_t string_find_last(string_t *str, const char *needle)
{
    assert(str != NULL && needle != NULL);

    size_t last_pos = SIZE_MAX;

    for (size_t i = 0; i < str->len; i++)
        if (!strncmp(str->data + i, needle, strlen(needle)))
            last_pos = i;

    return last_pos;
}

bool string_ends_with(string_t *str, const char *suffix)
{
    assert(str != NULL && suffix != NULL);

    size_t sf_len = strlen(suffix);

    return !strncmp(str->data + (str->len - sf_len), suffix, sf_len);
}

string_t string_substr(string_t *str, size_t pos, size_t n)
{
    string_t substr;
    string_init(&substr, n);

    size_t len = MIN(n, str->len - pos);
    memcpy(substr.data, str->data + pos, len + 1);

    substr.len = len;
    substr.cap = len + 1;

    return substr;
}

void string_insert(string_t *str, const char *lit, size_t pos)
{
    size_t lit_len = strlen(lit), n_len = str->len + lit_len, n_cap = str->cap;

    while (n_len > n_cap)
        n_cap = n_cap == 0 ? 1 : n_cap * 2;

    if (n_cap > str->cap)
        string_setcap(str, n_cap);

    if (pos < str->len - 1)
    {
        char *src = str->data + pos, *dest = src + lit_len;
        size_t bytes = str->len - pos;

        memmove(dest, src, bytes);
    }

    memmove(str->data + pos, lit, lit_len);
    str->len = n_len;
}

void string_format(string_t *str, const char *format, ...)
{
    va_list args;
    va_start(args, format);

    vsnprintf(str->data, str->cap, format, args);

    str->len = strlen(str->data);
}
