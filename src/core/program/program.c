#include <stdio.h>
#include <stdlib.h>
#include <sys/stat.h>

#include "core/diagnostic/reporter/cli_reporter.h"
#include "core/diagnostic/reporter/reporter.h"
#include "core/program/program.h"
#include "utils/control.h"
#include "utils/macros.h"
#include "utils/types/string.h"
#include "utils/types/string_view.h"

#ifdef _WIN32

#include <windows.h>
#define PATH_SEPARATOR '\\'

#else

#include <linux/limits.h>
#include <unistd.h>
#define PATH_SEPARATOR '/'

#endif

static bool is_regular_file(const char *path)
{
    struct stat buf;
    int status = stat(path, &buf);

    if (status != 0)
        return false;

    return S_ISREG(buf.st_mode);
}

static char *canonicalize_path(const char *path)
{
    char *resolved = NULL;

#ifdef _WIN32
    DWORD size = GetFullPathNameA(path, 0, NULL, NULL);
    if (size == 0)
        return NULL;

    resolved = malloc(size * sizeof(char));
    if (resolved == NULL)
        return NULL;

    if (GetFullPathNameA(path, size, resolved, NULL) == 0)
    {
        free(resolved);
        return NULL;
    }

    for (char *p = resolved; *p; p++)
        if (*p == '\\')
            *p = '/';

#else
    resolved = malloc(PATH_MAX * sizeof(char));
    if (resolved == NULL)
        return NULL;

    if (realpath(path, resolved) == NULL)
    {
        free(resolved);
        return NULL;
    }

#endif

    return resolved;
}

program_t program_from(const char *path)
{
    program_t program = (program_t){
        .path = canonicalize_path(path),
        .source = string_from(""),
        .lines = sv_vec_with_cap(16),
    };

    if (program.path == NULL)
        terminate("[program] failed to resolve file", EXIT_FAILURE);

    program_read_source(&program);

    return program;
}

void program_free(program_t *program)
{
    free((char *)program->path);

    string_free(&program->source);
    vec_free(&program->lines);
}

void program_read_source(program_t *program)
{
    if (!is_regular_file(program->path))
        terminate("[program_read] invalid file path provided", EXIT_FAILURE);

    clean(string_free) string_t path = string_from(program->path);
    if (!string_ends_with(&path, FILE_EXTENSION))
        terminate("[program_read] file must be a Hyacinth file (.hyc)",
                  EXIT_FAILURE);

    FILE *file = fopen(program->path, "r");
    if (file == NULL)
        terminate("[program_read] cannot open file", EXIT_FAILURE);

    fseek(file, 0, SEEK_END);
    size_t fsize = ftell(file);
    fseek(file, 0, 0);

    string_setcap(&program->source, fsize);
    size_t rsize = fread(program->source.data, sizeof(char), fsize, file);

    if (fsize != rsize)
        terminate("[program_read] failed to read the whole file", EXIT_FAILURE);

    size_t cursor = 0, line_start = cursor;
    while (cursor < fsize)
    {
        char c = program->source.data[cursor];
        if (c == '\n')
        {
            sv_vec_push(&program->lines, sv_from(&program->source, line_start,
                                                 cursor - line_start));

            line_start = cursor + 1;
        }

        cursor++;
    }

    if (line_start != cursor)
    {
        sv_vec_push(&program->lines,
                    sv_from(&program->source, line_start, cursor - line_start));
    }

    fclose(file);
}

void program_execute(program_t *program)
{
    //
    TODO("program_execute()")

    reporter_t *__reporter = reporter();

    *diag_vec_use(&__reporter->diagnostics) =
        (diagnostic_t){.severity = DIAG_ERROR,
                       .message = string_from("diagnostic test"),
                       .code = 123,
                       .range = {
                           (position_t){program, 1, 3, 0},
                           (position_t){program, 1, 6, 0},
                       }};

    __reporter->report(__reporter);
}
