#include <assert.h>
#include <stdlib.h>
#include <string.h>

#include "utils/control.h"
#include "utils/types/vector.h"

vector_t vec_with_size(size_t size, size_t e_size, vec_opts_t opts)
{
    vector_t vec = {.e_size = e_size, .opts = opts};
    vec_setcap(&vec, size);

    memset(vec.data, 0, size);

    return vec;
}

vector_t vec_with_cap(size_t cap, size_t e_size, vec_opts_t opts)
{
    vector_t vec = {.e_size = e_size, .opts = opts};
    vec_setcap(&vec, cap);

    return vec;
}

void vec_copy(vector_t *dest, vector_t *src) {}

void vec_move(vector_t *dest, vector_t *src)
{
    assert(dest != NULL && src != NULL);

    dest->size = src->size;
    dest->cap = src->cap;
    dest->e_size = src->e_size;

    dest->data = vec_reset(src);
}

void vec_free(vector_t *vec)
{
    assert(vec != NULL);

    if (vec->opts.destr)
        for (size_t i = 0; i < vec->size; i++)
            vec->opts.destr(vec_at(vec, i));

    free(vec_reset(vec));
}

void *vec_reset(vector_t *vec)
{
    assert(vec != NULL);

    void *buf = vec->data;

    vec->data = NULL;
    vec->size = 0;
    vec->cap = 0;
    vec->e_size = 0;

    return buf;
}

void vec_setcap(vector_t *vec, size_t cap)
{
    assert(vec != NULL && cap > 0);

    void *buf = realloc(vec->data, cap * vec->e_size);
    if (buf == NULL)
        terminate("[vector] failed to reallocate memory for the buffer of the "
                  "vector.",
                  EXIT_FAILURE);

    vec->data = buf;
    vec->cap = cap;

    if (vec->size > cap)
        vec->size = cap;
}

void *vec_at(vector_t *vec, size_t idx)
{
    assert(vec != NULL);

    if (idx >= vec->size)
        terminate("[vector] out of range index provided.", EXIT_FAILURE);

    return (char *)vec->data + (idx * vec->e_size);
}

void *vec_silent_at(vector_t *vec, size_t idx)
{
    assert(vec != NULL);

    if (idx >= vec->size)
        return NULL;

    return (char *)vec->data + (idx * vec->e_size);
}

void vec_insert_el(vector_t *vec, void *el, size_t pos)
{
    assert(vec != NULL && pos >= vec->size);

    if (vec->size >= vec->cap)
        vec_setcap(vec, vec->cap * 2);

    if (pos < vec->size)
    {
        void *src = (char *)vec->data + (pos * vec->e_size);
        void *dest = (char *)src + (vec->e_size); // Shift by a single element

        size_t bytes = (vec->size - pos) * vec->e_size;

        memmove(dest, src, bytes);
    }

    memmove((char *)vec->data + (pos * vec->e_size), el, vec->e_size);
    vec->size++;
}

void vec_insert_vec(vector_t *vec, vector_t *other, size_t in_pos, size_t f_pos,
                    size_t n)
{
    assert(vec != NULL && other != NULL);

    if (vec->e_size != other->e_size)
        terminate("[vec_insert] cannot insert vector of different element size",
                  EXIT_FAILURE);

    if (in_pos > vec->size)
        terminate("[vec_insert_vec] insert position is out of bounds",
                  EXIT_FAILURE);

    if (f_pos >= other->size)
        terminate("[vec_insert_vec] starting position of the other vector is "
                  "out of bounds",
                  EXIT_FAILURE);

    size_t n_size = vec->size + n;
    size_t n_cap = vec->cap;

    while (n_size > n_cap)
        n_cap = n_cap == 0 ? 1 : n_cap * 2;

    if (n_cap > vec->cap)
        vec_setcap(vec, n_cap);

    // If the position is less than the size
    // The vector will be inserted in the middle, therefore, the data must be
    // shifted
    if (in_pos < vec->size)
    {
        void *src = (char *)vec->data + (in_pos * vec->e_size);
        void *dest = (char *)src + (n * vec->e_size);

        size_t bytes = (vec->size - in_pos) * vec->e_size;

        memmove(dest, src, bytes);
    }

    void *target = (char *)vec->data + (in_pos * vec->e_size);
    memmove(target, (char *)other->data + (f_pos * vec->e_size),
            n * other->e_size);

    vec->size = n_size;
}

void vec_insert_full_vec(vector_t *vec, vector_t *other, size_t pos)
{
    vec_insert_vec(vec, other, pos, 0, other->size);
}

size_t vec_push(vector_t *vec, void *element)
{
    assert(vec != NULL);

    vec_insert_el(vec, element, vec->size);
    return vec->size;
}

void vec_pop(vector_t *vec)
{
    assert(vec != NULL);

    if (vec->size < 1)
        terminate("[vector] cannot pop an empty vector.", EXIT_FAILURE);

    if (vec->opts.destr)
        vec->opts.destr(vec_at(vec, vec->size - 1));

    vec->size--;
}

void vec_silent_pop(vector_t *vec)
{
    assert(vec != NULL);
    if (vec->size < 1)
        return;

    if (vec->opts.destr)
        vec->opts.destr(vec_at(vec, vec->size - 1));

    vec->size--;
}

void *vec_use(vector_t *vec)
{
    assert(vec != NULL);

    if (vec->size >= vec->cap)
        vec_setcap(vec, vec->cap * 2);

    void *target = (char *)vec->data + (vec->size * vec->e_size);
    vec->size++;

    return target;
}
