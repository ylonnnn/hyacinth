// For non-testing executions
#include "core/program/program.h"

// For test executions
#include "testing/testing.h"
// #include "utils/macros.h"
// #include "utils/types/hashmap.h"
// #include <stdio.h>
// #include <string.h>

// size_t str_hash(void *key)
// {
//     string_t *str = key;
//     size_t hash = 0;

//     for (size_t i = 0; i < str->len; i++)
//         hash += *string_at(str, i);

//     return hash;
// }

// bool str_eq(void *a, void *b)
// {
//     return string_equal((string_t *)a, (string_t *)b);
// }

// T_HASHMAP(string_t, int, test_hm,
//           ((hashmap_opts_t){
//               str_hash, str_eq,
//               (hashmap_data_opts_t){
//                   sizeof(string_t), (hm_data_destr)string_free,
//                   (hm_data_cp)string_copy_to, (hm_data_mv)string_move, NULL},
//               (hashmap_data_opts_t){sizeof(int), NULL, NULL, NULL}}))

void handle(const char *path)
{
#ifdef __testing
    begin_tests("hyc/tests");

    // clean(hashmap_free) hashmap_t hm = test_hm_with_cap(8);

    // HM_INSERT(hm, string_from("somelongstring"), 1923, true);
    // HM_INSERT(hm, string_from("apple"), 10, true);

    // int *res;
    // HM_FIND(int, res, hm, string_from("somelongstring"), true);

    // if (res)
    //     printf("res: %d\n", *res);
    // else
    //     printf("res: NULL\n");

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
