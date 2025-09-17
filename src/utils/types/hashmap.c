#include <assert.h>
#include <stdlib.h>
#include <string.h>

#include "utils/control.h"
#include "utils/types/hashmap.h"

static void *hm_default_allocator([[maybe_unused]] void *src, size_t size)
{
    return malloc(size);
}

hashmap_entry_t *hm_entry_alloc(hashmap_t *hashmap, void *key, void *value,
                                bool move)
{
    assert(hashmap != NULL);

    hashmap_entry_t *entry = malloc(sizeof(hashmap_entry_t));
    if (entry == NULL)
        terminate("[hm_entry_alloc] failed to allocate memory for a new "
                  "hashmap entry",
                  EXIT_FAILURE);

    hashmap_data_opts_t *k_opts = &hashmap->opts.key,
                        *v_opts = &hashmap->opts.val;

    entry->key = k_opts->alloc(key, k_opts->size);
    if (entry->key == NULL)
    {
        free(entry);
        terminate("[hm_entry_alloc] failed to allocate memory for the hashmap "
                  "entry key",
                  EXIT_FAILURE);
    }

    if (move && k_opts->mv)
        k_opts->mv(entry->key, key);
    else if (k_opts->cp)
        k_opts->cp(key, entry->key);
    else
        memcpy(entry->key, key, k_opts->size);

    entry->value = v_opts->alloc(value, v_opts->size);
    if (entry->value == NULL)
    {
        free(entry);
        terminate("[hm_entry_alloc] failed to allocate memory for the hashmap "
                  "entry value",
                  EXIT_FAILURE);
    }

    if (move && v_opts->mv)
        v_opts->mv(entry->value, value);
    else if (v_opts->cp)
        v_opts->cp(value, entry->value);
    else
        memcpy(entry->value, value, v_opts->size);

    return entry;
}

void hm_entry_free(hashmap_t *hashmap, hashmap_entry_t *hm_entry)
{
    assert(hashmap != NULL);
    assert(hm_entry != NULL);

    hashmap_data_opts_t *k_opts = &hashmap->opts.key,
                        *v_opts = &hashmap->opts.val;

    // Destroy the key
    if (k_opts->destr)
        k_opts->destr(hm_entry->key);

    // Free the key buffer
    free(hm_entry->key);

    // Destroy the value
    if (v_opts->destr)
        v_opts->destr(hm_entry->value);

    // Free the value buffer
    free(hm_entry->value);

    // Free the entry buffer
    free(hm_entry);
}

hashmap_t hashmap_with_cap(size_t cap, hashmap_opts_t opts)
{
    assert(opts.hash != NULL);
    assert(opts.eq != NULL);

    hashmap_t hashmap = {
        .entries = NULL,
        .size = 0,
        .cap = cap,
        .opts = opts,
    };

    hm_data_alloc k_alloc = hashmap.opts.key.alloc,
                  v_alloc = hashmap.opts.val.alloc;

    if (!k_alloc)
        hashmap.opts.key.alloc = hm_default_allocator;

    if (!v_alloc)
        hashmap.opts.val.alloc = hm_default_allocator;

    hashmap_entry_t **buf = calloc(cap, sizeof(hashmap_entry_t *));
    if (buf == NULL)
        terminate(
            "[hashmap_with_cap] failed to allocate memory for the hashmap "
            "entry buffer",
            EXIT_FAILURE);

    hashmap.entries = buf;

    return hashmap;
}

void hashmap_free(hashmap_t *hashmap)
{
    assert(hashmap != NULL);

    hashmap_data_opts_t *k_opts = &hashmap->opts.key,
                        *v_opts = &hashmap->opts.val;

    for (size_t i = 0; i < hashmap->cap; i++)
    {
        hashmap_entry_t *entry = hashmap->entries[i];
        if (entry == NULL)
            continue;

        while (entry != NULL)
        {
            hashmap_entry_t *prev = entry;
            entry = entry->next;

            // Free the entry
            hm_entry_free(hashmap, prev);
        }
    }

    free(hashmap->entries);
}

double hashmap_load_factor(hashmap_t *hashmap)
{
    return (double)(hashmap->size) / hashmap->cap;
}

void hashmap_rehash(hashmap_t *hashmap)
{
    assert(hashmap != NULL);

    if (hashmap_load_factor(hashmap) < HASHMAP_REHASH_THRESHOLD)
        return;

    printf("rehashing...\n");

    size_t n_cap = hashmap->cap * 2;
    hashmap_entry_t **buf = calloc(n_cap, sizeof(hashmap_entry_t *));

    if (buf == NULL)
        terminate("[hashmap_rehash] failed to allocate memory for the new "
                  "hashmap entry buffer",
                  EXIT_FAILURE);

    for (size_t i = 0; i < hashmap->cap; i++)
    {
        hashmap_entry_t *curr = hashmap->entries[i];
        if (curr == NULL)
            continue;

        while (curr != NULL)
        {
            size_t idx = hashmap->opts.hash(curr->key) % n_cap;
            hashmap_entry_t *entry = buf[idx], *p_next = curr->next;

            curr->next = entry;
            buf[idx] = curr;

            curr = p_next;
        }
    }

    free(hashmap->entries);

    hashmap->entries = buf;
    hashmap->cap = n_cap;

    printf("rehash complete\n");
}

void hashmap_insert(hashmap_t *hashmap, void *key, void *value, bool move)
{
    assert(hashmap != NULL);

    hashmap_rehash(hashmap);

    size_t idx = hashmap->opts.hash(key) % hashmap->cap;
    hashmap_entry_t *entry = hashmap->entries[idx],
                    *n_entry = hm_entry_alloc(hashmap, key, value, move),
                    *curr = entry;

    while (curr != NULL)
    {
        // If an existing key is found, replace the entry
        if (hashmap->opts.eq(key, &curr->key))
        {
            n_entry->next = entry;

            hashmap->entries[idx] = n_entry;
            hashmap->size++;

            return;
        }

        curr = curr->next;
    }

    // Otherwise, append to the linked list as the head
    n_entry->next = entry;

    hashmap->entries[idx] = n_entry;
    hashmap->size++;
}

void *hashmap_find(hashmap_t *hashmap, void *key, bool destr_key)
{
    assert(hashmap != NULL);

    size_t idx = hashmap->opts.hash(key) % hashmap->cap;
    hashmap_entry_t *entry = hashmap->entries[idx];

    while (entry != NULL)
    {
        if (hashmap->opts.eq(key, entry->key))
        {
            if (destr_key && hashmap->opts.key.destr)
                hashmap->opts.key.destr(key);

            return entry->value;
        }

        entry = entry->next;
    }

    if (destr_key && hashmap->opts.key.destr)
        hashmap->opts.key.destr(key);

    return NULL;
}
