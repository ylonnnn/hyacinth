#ifndef __UTILS_TYPES_HASHMAP_H__
#define __UTILS_TYPES_HASHMAP_H__

#include <stdbool.h>
#include <stddef.h>

#define HASHMAP_REHASH_THRESHOLD 0.75

typedef void (*hm_data_destr)(void *self);
typedef void (*hm_data_cp)(void *self, void *dest);
typedef void (*hm_data_mv)(void *dest, void *self);

typedef struct hashmap_data_opts
{
    size_t size;
    hm_data_destr destr;
    hm_data_cp cp;
    hm_data_mv mv;
} hashmap_data_opts_t;

typedef size_t (*hm_hash_fn)(void *key);
typedef bool (*hm_eq_fn)(void *a, void *b);

typedef struct hashmap_entry
{
    void *key;
    void *value;

    struct hashmap_entry *next;
} hashmap_entry_t;

typedef struct hashmap hashmap_t;

// Constructors
hashmap_entry_t *hm_entry_alloc(hashmap_t *hashmap, void *key, void *value,
                                bool move);

// Destructors
void hm_entry_free(hashmap_t *hashmap, hashmap_entry_t *hm_entry);

typedef struct hashmap_opts
{
    hm_hash_fn hash;
    hm_eq_fn eq;

    hashmap_data_opts_t key;
    hashmap_data_opts_t val;
} hashmap_opts_t;

typedef struct hashmap
{
    hashmap_entry_t **entries;
    size_t cap, size;
    hashmap_opts_t opts;
} hashmap_t;

// Constructors
hashmap_t hashmap_with_cap(size_t cap, hashmap_opts_t opts);

// Destructors
void hashmap_free(hashmap_t *hashmap);

// Helper

double hashmap_load_factor(hashmap_t *hashmap);
void hashmap_rehash(hashmap_t *hashmap);

void hashmap_insert(hashmap_t *hashmap, void *key, void *value, bool move);
void *hashmap_find(hashmap_t *hashmap, void *key, bool destr_key);

#define T_HM_CONSTR(K, V, fn, opts)                                            \
    static inline hashmap_t fn##_with_cap(size_t cap)                          \
    {                                                                          \
        return hashmap_with_cap(cap, opts);                                    \
    }

#define HM_INSERT(hashmap, key, value, move)                                   \
    {                                                                          \
        typeof(key) lv = key;                                                  \
        hashmap_insert(&hashmap, &lv, &value, move)                            \
    }

#define T_HM_INSERT(K, V, fn)                                                  \
    static inline void fn##_insert(hashmap_t *hashmap, K key, V value,         \
                                   bool move)                                  \
    {                                                                          \
        hashmap_insert(hashmap, &key, &value, move);                           \
    }

#define HM_FIND(V, res, hashmap, key, destr_key)                               \
    {                                                                          \
        typeof(key) lv = key;                                                  \
        res = (V *)hashmap_find(&hashmap, &lv, destr_key);                     \
    }

#define T_HM_FIND(K, V, fn)                                                    \
    static inline V *fn##_find(hashmap_t *hashmap, K key, bool destr_key)      \
    {                                                                          \
        return (V *)hashmap_find(hashmap, &key, destr_key);                    \
    }

#endif
