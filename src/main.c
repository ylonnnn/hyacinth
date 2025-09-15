#include "core/program/program.h"
#include "testing/testing.h"
#include "utils/macros.h"
#include "utils/types/hashmap.h"
#include "utils/types/string.h"
#include <stdio.h>

size_t str_hash(void *key)
{
    string_t *str = key;
    size_t hash = 0;

    for (size_t i = 0; i < str->len; i++)
    {
        char c = *string_at(str, i);
        hash += c;
    }

    return hash;
}

bool str_eq(void *a, void *b)
{
    return string_equal((string_t *)a, (string_t *)b);
}

typedef enum type
{
    keyword_let = 69,
    keyword_fn,
} type_t;

T_HM_CONSTR(string_t, type_t, se_hm,
            ((hashmap_opts_t){
                str_hash, str_eq,
                (hashmap_data_opts_t){
                    sizeof(string_t), (hm_data_destr)string_free,
                    (hm_data_cp)string_copy_to, (hm_data_mv)string_move},
                (hashmap_data_opts_t){sizeof(type_t), NULL, NULL, NULL}}))
T_HM_INSERT(string_t, type_t, se_hm)
T_HM_FIND(string_t, type_t, se_hm)

void handle(const char *path)
{
#ifdef __testing
    // begin_tests("hyc/tests");

    clean(hashmap_free) hashmap_t hashmap = se_hm_with_cap(8);

    se_hm_insert(&hashmap, string_from("let"), keyword_let, true);
    se_hm_insert(&hashmap, string_from("fn"), keyword_fn, true);

    type_t *tl;
    HM_FIND(type_t, tl, hashmap, string_from("let"), true);

    // // type_t *tl = se_hm_find(&hashmap, string_from("let"), true);
    // // type_t *tf = se_hm_find(&hashmap, string_from("fn"), true);
    // // type_t *ne = se_hm_find(&hashmap, string_from("struct"), true);

    printf("tl: %d\n", (tl == NULL ? 0 : *tl));
    // printf("tf: %d\n", (tf == NULL ? 0 : *tf));
    // printf("ne: %p\n", ne);

#else
#include "utils/macros.h"

    clean(program_free) program_t program = program_from(path);
    program_execute(&program);

#endif
}

int main(int argc, char **argv)
{
    handle(argc < 2 ? "hyc/../hyc/tests/../../hyc/tests/main.hyc" : argv[1]);

    return 0;
}
