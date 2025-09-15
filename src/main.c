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

T_HM_CONSTR(string_t, int, se_hm,
            ((hashmap_opts_t){
                str_hash, str_eq,
                (hashmap_data_opts_t){
                    sizeof(string_t), (hm_data_destr)string_free,
                    (hm_data_cp)string_copy_to, (hm_data_mv)string_move},
                (hashmap_data_opts_t){sizeof(int), NULL, NULL, NULL}}))
T_HM_INSERT(string_t, int, se_hm)
T_HM_FIND(string_t, int, se_hm)

void handle(const char *path)
{
#ifdef __testing
    // begin_tests("hyc/tests");

    clean(hashmap_free) hashmap_t hashmap = se_hm_with_cap(8);

    se_hm_insert(&hashmap, string_from("000"), 1, true);
    se_hm_insert(&hashmap, string_from("001"), 1, true);
    se_hm_insert(&hashmap, string_from("002"), 2, true);
    se_hm_insert(&hashmap, string_from("003"), 3, true);
    se_hm_insert(&hashmap, string_from("004"), 4, true);
    se_hm_insert(&hashmap, string_from("005"), 5, true);
    se_hm_insert(&hashmap, string_from("005"), 5, true);
    se_hm_insert(&hashmap, string_from("006"), 6, true);
    se_hm_insert(&hashmap, string_from("007"), 7, true);
    se_hm_insert(&hashmap, string_from("008"), 8, true);
    se_hm_insert(&hashmap, string_from("009"), 9, true);
    se_hm_insert(&hashmap, string_from("010"), 10, true);
    se_hm_insert(&hashmap, string_from("011"), 11, true);

    for (int i = 0; i < 12; i++)
    {
        char key[6];

        if (i < 10)
            snprintf(key, 6, "00%d", i);
        else
            snprintf(key, 6, "0%d", i);

        printf("key: %s\n", key);

        int *val = NULL;
        HM_FIND(int, val, hashmap, string_from(key), true);

        if (val)
            printf("val: %d\n", *val);
        else
            printf("val: NULL\n");
    }

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
