#include <assert.h>
#include <string.h>

#include "utils/types/string_view.h"

string_view_t sv_from(string_t *str, size_t start, size_t n)
{
    return (string_view_t){
        .data = str->data + start,
        .len = n,
    };
}

string_view_t sv_from_sv(string_view_t *sv, size_t start, size_t n)
{
    return (string_view_t){
        .data = sv->data + start,
        .len = n,
    };
}

string_view_t sv_copy(string_view_t *sv)
{
    return (string_view_t){
        .data = sv->data,
        .len = sv->len,
    };
}

void sv_copy_to(string_view_t *sv, string_view_t *dest) { *dest = sv_copy(sv); }

string_view_t sv_substr(string_view_t *sv, size_t pos, size_t n)
{
    return sv_from_sv(sv, pos, n);
}

bool sv_equal(string_view_t *a, string_view_t *b)
{
    if (a->len != b->len)
        return false;

    return strncmp(a->data, b->data, a->len);
}

void sv_print(string_view_t *sv, FILE *stream)
{
    assert(sv != NULL);

    fwrite(sv->data, sizeof(char), sv->len, stream);
}
