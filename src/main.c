#include <stdio.h>

#include "core/program/program.h"
#include "utils/macros.h"
#include "utils/types/string.h"

// #define TEST

void execute(const char *path)
{
    clean(program_free) program_t program = program_from(path);

    program_execute(&program);
}

T_VEC_CONSTR(int, int_vec, (vec_opts_t){NULL})
T_VEC_PUSH(int, int_vec_push)
T_VEC_AT(int, int_vec_at)

void test()
{
    string_t str = string_from("Hello, World");
    printf("%zu bytes\n", str.len);

    string_insert(&str, "ALongerStringLiteral", 5);

    printf("%s\n", str.data);
    printf("%zu bytes\n", str.len);
}

int main(int argc, char **argv)
{
#ifdef TEST
    test();

#else
    const char *path =
        argc < 2 ? "hyc/../hyc/tests/../../hyc/tests/main.hyc" : argv[1];
    execute(path);

#endif

    return 0;
}
