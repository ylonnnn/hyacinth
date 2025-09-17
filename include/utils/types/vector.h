#ifndef __UTILS_TYPES_VECTOR_H__
#define __UTILS_TYPES_VECTOR_H__

#include <stdbool.h>
#include <stdlib.h>

typedef void (*vec_el_destr)(void *el);
typedef void (*vec_el_cp)(void *el, void *dest);
typedef void (*vec_el_mv)(void *dest, void *el);

typedef struct vec_opts
{
    vec_el_destr destr; // Vector Element Destructor
    vec_el_cp cp;       // Vector Element Copy Constructor
    vec_el_mv mv;       // Vector Element Move Constructor
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

vector_t vec_copy(vector_t *vec);
void vec_copy_to(vector_t *vec, vector_t *dest);
void vec_move(vector_t *dest, vector_t *src);

// Destructor
void vec_free(vector_t *vec);

// Helper

/**
 * Resets the provided vector and returns its allocated buffer.
 */
void *vec_reset(vector_t *vec);
void vec_setcap(vector_t *vec, size_t cap);

void vec_resize(vector_t *vec, size_t n_size);

void *vec_at(vector_t *vec, size_t idx);

/**
 * Returns the element at the given `idx`. If the index provided is out of
 * bounds, it will fail silently and return `NULL`.
 */
void *vec_silent_at(vector_t *vec, size_t idx);

void *vec_back(vector_t *vec);

void vec_insert_el(vector_t *vec, void *el, size_t pos, bool move);
void vec_insert_vec(vector_t *vec, vector_t *other, size_t in_pos, size_t f_pos,
                    size_t n, bool move);
void vec_insert_full_vec(vector_t *vec, vector_t *other, size_t pos, bool move);

size_t vec_push(vector_t *vec, void *element);
size_t vec_mpush(vector_t *vec, void *element);
void vec_pop(vector_t *vec);
void vec_silent_pop(vector_t *vec);

void *vec_use(vector_t *vec);

// Macros
#define T_VEC(T, n, opts)                                                      \
    static inline vector_t n##_with_size(size_t size)                          \
    {                                                                          \
        return vec_with_size(size, sizeof(T), opts);                           \
    }                                                                          \
    static inline vector_t n##_with_cap(size_t cap)                            \
    {                                                                          \
        return vec_with_cap(cap, sizeof(T), opts);                             \
    }                                                                          \
    static inline T *n##_reset(vector_t *vec) { return (T *)vec_reset(vec); }  \
    static inline T *n##_at(vector_t *vec, size_t idx)                         \
    {                                                                          \
        return (T *)vec_at(vec, idx);                                          \
    }                                                                          \
    static inline T *n##_silent_at(vector_t *vec, size_t idx)                  \
    {                                                                          \
        return (T *)vec_silent_at(vec, idx);                                   \
    }                                                                          \
    static inline T *n##_back(vector_t *vec) { return (T *)vec_back(vec); }    \
    static inline void n##_insert_el(vector_t *vec, T el, size_t pos,          \
                                     bool move)                                \
    {                                                                          \
        return vec_insert_el(vec, &el, pos, move);                             \
    }                                                                          \
    static inline size_t n##_push(vector_t *vec, T el)                         \
    {                                                                          \
        return vec_push(vec, &el);                                             \
    }                                                                          \
    static inline size_t n##_mpush(vector_t *vec, T el)                        \
    {                                                                          \
        return vec_mpush(vec, &el);                                            \
    }                                                                          \
    static inline T *n##_use(vector_t *vec) { return (T *)vec_use(vec); }

#define VEC_RESET(T, vec) (T *)vec_reset(&vec)

#define VEC_AT(T, vec, idx) (T *)vec_at(&vec, idx)

#define VEC_INSERT_EL(vec, el, pos, move)                                      \
    {                                                                          \
        typeof(el) lv = el;                                                    \
        vec_insert_el(&vec, &lv, pos, move);                                   \
    }

#define VEC_PUSH(vec, el)                                                      \
    {                                                                          \
        typeof(el) lv = el;                                                    \
        vec_push(&vec, &lv);                                                   \
    }

#define VEC_MPUSH(vec, el)                                                     \
    {                                                                          \
        typeof(el) lv = el;                                                    \
        vec_mpush(&vec, &lv);                                                  \
    }

#define VEC_USE(vec, value) *(typeof(value) *)vec_use(&vec) = value;

#endif
