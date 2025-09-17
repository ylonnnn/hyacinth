#include <stdio.h>
#include <stdlib.h>
#include <sys/stat.h>
#include <time.h>

#include "core/diagnostic/diagnostic.h"
#include "core/diagnostic/reporter/cli_reporter.h"
#include "core/diagnostic/reporter/reporter.h"
#include "core/program/program.h"
#include "core/result/result.h"
#include "lexer/lexer.h"
#include "lexer/token.h"
#include "utils/control.h"
#include "utils/macros.h"
#include "utils/style.h"
#include "utils/time.h"
#include "utils/types/string.h"
#include "utils/types/string_view.h"
#include "utils/types/vector.h"

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

void program_initialize_lexer(program_t *program)
{
    lexer_t *buf = malloc(sizeof(lexer_t));
    if (buf == NULL)
        terminate("[program_from] failed to allocate memory for program lexer",
                  EXIT_FAILURE);

    *buf = lexer_from(program);
    program->lexer = buf;
}

program_t program_from(const char *path)
{
    program_t program = (program_t){
        .path = canonicalize_path(path),
        .lines = sv_vec_with_cap(16),
        .lexer = NULL,
    };

    if (program.path == NULL)
        terminate("[program_from] failed to resolve file", EXIT_FAILURE);

    program_read_source(&program);

    return program;
}

void program_free(program_t *program)
{
    free((char *)program->path);

    string_free(&program->source);
    vec_free(&program->lines);

    lexer_free(program->lexer);
    free(program->lexer);
}

void program_read_source(program_t *program)
{
    if (!is_regular_file(program->path))
        terminate("[program_read] invalid file path provided", EXIT_FAILURE);

    clean(string_free) string_t path = string_from(program->path);
    if (!string_ends_with(&path, FILE_EXT))
        terminate("[program_read] file must be a Hyacinth file (.hyc)",
                  EXIT_FAILURE);

    FILE *file = fopen(program->path, "r");
    if (file == NULL)
        terminate("[program_read] cannot open file", EXIT_FAILURE);

    fseek(file, 0, SEEK_END);
    size_t fsize = ftell(file);
    fseek(file, 0, 0);

    if (fsize == 0)
        terminate("[program_read] cannot read from an empty file",
                  EXIT_FAILURE);

    string_init(&program->source, fsize);
    size_t rsize = fread(program->source.data, sizeof(char), fsize, file);

    if (fsize != rsize)
        terminate("[program_read] failed to read the whole file", EXIT_FAILURE);

    program->source.len = rsize - 1;
    program->source.data[program->source.len] = '\0';

    size_t cursor = 0, line_start = cursor;

    while (cursor < fsize)
    {
        char c = program->source.data[cursor];
        if (c == '\n')
        {
            string_view_t sv =
                sv_from(&program->source, line_start, cursor - line_start);
            VEC_PUSH(program->lines, sv);

            line_start = cursor + 1;
        }

        cursor++;
    }

    if (line_start != cursor)
    {
        VEC_PUSH(program->lines,
                 sv_from(&program->source, line_start, cursor - line_start));
    }

    fclose(file);
}

result_t program_lex(program_t *program)
{
    program_initialize_lexer(program);
    return *lexer_tokenize(program->lexer);
}

void program_lex_wres(program_t *program, result_t *p_res)
{
    program_lex(program);

    result_t *l_res = &program->lexer->result;
    result_adapt(p_res, l_res->status, &l_res->diagnostics);
}

void program_execute(program_t *program)
{
    //
    TODO("program_execute()")

    clean(result_free) result_t program_result = result_create(RES_SUCCESS);

    bool succeeded = true;
    double start = get_nanosec();

    // Lexer
    program_lex_wres(program, &program_result);
    succeeded = program_result.status == RES_SUCCESS;

    // Parser

    double end = get_nanosec();

    vector_t *tokens = &program->lexer->tokens;
    for (size_t i = 0; i < tokens->size; i++)
    {
        token_t *token = token_vec_at(tokens, i);
        token_print(token);
        puts("");
    }

    reporter_t *__reporter = reporter();
    vec_move(&__reporter->diagnostics, &program_result.diagnostics);
    // vec_insert_full_vec(&__reporter->diagnostics,
    // &program_result.diagnostics,
    //                     __reporter->diagnostics.size, true);

    report_result_t res = __reporter->report(__reporter);

    text_style color = succeeded ? C_GREEN : C_RED;
    printf("\t%s%u %s[%si%s]%s, %s%u %s[%s~%s]%s, %s%u "
           "%s[%s*%s]%s\n\t%s[%s%s%s]%s Program Executed %s(%s%g ms%s)%s\n\n",
           INFO_COLOR, res.diag_count[DIAG_INFO], C_BRIGHT_BLACK, INFO_COLOR,
           C_BRIGHT_BLACK, S_RESET, WARN_COLOR, res.diag_count[DIAG_WARN],
           C_BRIGHT_BLACK, WARN_COLOR, C_BRIGHT_BLACK, S_RESET, ERROR_COLOR,
           res.diag_count[DIAG_ERROR], C_BRIGHT_BLACK, ERROR_COLOR,
           C_BRIGHT_BLACK, S_RESET, C_BRIGHT_BLACK, color,
           succeeded ? "/" : "X", C_BRIGHT_BLACK, S_RESET, C_BRIGHT_BLACK,
           color, (end - start) / NANOSECONDS_PER_MILLISECOND, C_BRIGHT_BLACK,
           S_RESET);
}
