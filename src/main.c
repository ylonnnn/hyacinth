#include "core/diagnostic/diagnostic.h"
#include "core/program/program.h"
#include "testing/testing.h"
#include "utils/macros.h"
#include "utils/types/string.h"
#include "utils/types/vector.h"
#include <stdio.h>

void handle(const char *path)
{
#ifdef __testing
    begin_tests("hyc/tests");

    // clean(vec_free) vector_t strs = str_vec_with_cap(8),
    //                          strs_b = str_vec_with_cap(strs.cap);

    // // *str_vec_use(&strs) = string_from("hello world");
    // // *str_vec_use(&strs) = string_from("another string");
    // // *str_vec_use(&strs) = string_from("test blabla");

    // str_vec_mpush(&strs, string_from("hello world"));
    // str_vec_mpush(&strs, string_from("another string"));
    // str_vec_mpush(&strs, string_from("test blabla"));

    // // VEC_PUSH(strs, string_from("hello world"));
    // // VEC_PUSH(strs, string_from("another string"));
    // // VEC_PUSH(strs, string_from("test blabla"));

    // vec_insert_full_vec(&strs_b, &strs, strs_b.size, true);

    // printf("strs: %zu\nstrs_b: %zu\n", strs.size, strs_b.size);
    // for (size_t i = 0; i < (MAX(strs.size, strs_b.size)); i++)
    // {
    //     string_t *s = str_vec_silent_at(&strs, i),
    //              *s_b = str_vec_silent_at(&strs_b, i);

    //     printf("s: %s\ns_b: %s\n", (s == NULL ? NULL : s->data),
    //            (s_b == NULL ? NULL : s_b->data));
    // }

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
