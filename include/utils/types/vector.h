#ifndef __UTILS_TYPES_VECTOR_H__
#define __UTILS_TYPES_VECTOR_H__

#include <stdlib.h>

typedef void (*vec_el_destr)(void *el);
// typedef void (*vec_el_mv)(void *el);
// typedef void (*vec_el_cp)(void *el);

typedef struct vec_opts
{
    vec_el_destr destr; // Vector Element Destructor
    // vec_el_mv mv;       // Vector Element Move Constructor
    // vec_el_cp cp;       // Vector Element Copy Constructor
} vec_opts_t;

typedef struct vector
{
    void *data;
    size_t cap, size, e_size;
    vec_opts_t opts;
} vector_t;

// Constructors
vector_t vec_with_size(size_t size, size_t e_size, vec_opts_t opts);
vector_t vec_with_cap(size_t cap, size_t e_size, vec_opts_t opts);

void vec_move(vector_t *dest, vector_t *src);

// Destructor
void vec_free(vector_t *vec);

// Helper

/**
 * Resets the provided vector and returns its allocated buffer.
 */
void *vec_reset(vector_t *vec);
void vec_setcap(vector_t *vec, size_t cap);

void *vec_at(vector_t *vec, size_t idx);

/**
 * Returns the element at the given `idx`. If the index provided is out of
 * bounds, it will fail silently and return `NULL`.
 */
void *vec_silent_at(vector_t *vec, size_t idx);

void vec_insert_el(vector_t *vec, void *el, size_t pos);
void vec_insert_vec(vector_t *vec, vector_t *other, size_t in_pos, size_t f_pos,
                    size_t n);
void vec_insert_full_vec(vector_t *vec, vector_t *other, size_t pos);

size_t vec_push(vector_t *vec, void *element);
void vec_pop(vector_t *vec);
void vec_silent_pop(vector_t *vec);

void *vec_use(vector_t *vec);

// Macros
#define T_VEC_CONSTR(T, fn, opts)                                              \
    static inline vector_t fn##_with_size(size_t size)                         \
    {                                                                          \
        return vec_with_size(size, sizeof(T), opts);                           \
    }                                                                          \
    static inline vector_t fn##_with_cap(size_t cap)                           \
    {                                                                          \
        return vec_with_cap(cap, sizeof(T), opts);                             \
    }

#define VEC_RESET(T, vec) (T *)vec_reset(&vec)
#define T_VEC_RESET(T, fn)                                                     \
    static inline T *fn(vector_t *vec) { return (T *)vec_reset(vec); }

#define VEC_AT(T, vec, idx) (T *)vec_at(&vec, idx)
#define T_VEC_AT(T, fn)                                                        \
    static inline T *fn(vector_t *vec, size_t idx)                             \
    {                                                                          \
        return (T *)vec_at(vec, idx);                                          \
    }

#define VEC_INSERT_EL(vec, el, pos)                                            \
    {                                                                          \
        typeof(el) lv = el;                                                    \
        vec_insert_el(&vec, &lv, pos);                                         \
    }

#define T_VEC_INSERT(T, fn)                                                    \
    static inline void fn##_insert_el(vector_t *vec, T el, size_t pos)         \
    {                                                                          \
        return vec_insert_el(vec, &el, pos);                                   \
    }

#define VEC_PUSH(vec, el)                                                      \
    {                                                                          \
        typeof(el) lv = el;                                                    \
        vec_push(&vec, &lv);                                                   \
    }

#define T_VEC_PUSH(T, fn)                                                      \
    static inline size_t fn(vector_t *vec, T el) { return vec_push(vec, &el); }

#define VEC_USE(vec, value) *(typeof(value) *)vec_use(&vec) = value;
#define T_VEC_USE(T, fn)                                                       \
    static inline T *fn(vector_t *vec) { return (T *)vec_use(vec); }

#endif
