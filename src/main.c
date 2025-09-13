#include "testing/testing.h"

void handle(const char *path)
{
#ifdef __testing
    begin_tests("hyc/tests");

#else
#include "core/program/program.h"
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
