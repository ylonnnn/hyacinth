#include "core/program/program.h"
#include "utils/macros.h"

void execute(const char *path)
{
    clean(program_free) program_t program = program_from(path);

    program_execute(&program);
}

int main(int argc, char **argv)
{
    const char *path =
        argc < 2 ? "hyc/../hyc/tests/../../hyc/tests/main.hyc" : argv[1];
    execute(path);

    return 0;
}
