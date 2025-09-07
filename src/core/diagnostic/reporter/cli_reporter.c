#include <stdio.h>
#include <string.h>

#include "core/diagnostic/diagnostic.h"
#include "core/diagnostic/reporter/cli_reporter.h"
#include "core/diagnostic/reporter/reporter.h"
#include "core/position/position.h"
#include "utils/control.h"
#include "utils/macros.h"
#include "utils/style.h"
#include "utils/types/string.h"
#include "utils/types/string_view.h"

#ifdef _WIN32
#include <direct.h>
#include <shlwapi.h>
#include <windows.h>
#define getcwd _getcwd
#define dirname(path) PathRemoveFileSpec(path)
#else
#include <libgen.h>
#include <unistd.h>
#endif

static bool initialized = false;

static const char *color_of(diagnostic_severity_t severity)
{
    const text_style colors[] = {INFO_COLOR, WARN_COLOR, ERROR_COLOR};
    return colors[severity];
}

static const char *severity_to_string(diagnostic_severity_t severity)
{
    const char *sev_str[] = {"info", "warning", "error"};
    return sev_str[severity];
}

static const char *severity_prefix(diagnostic_severity_t severity)
{
    const char *sev_pref[] = {"I0", "W0", "E0"};
    return sev_pref[severity];
}

static string_t point_position_range(diagnostic_severity_t severity,
                                     position_range_t *range)
{
    position_t *start = &range->start, *end = &range->end;
    program_t *program = start->program;

    vector_t *p_lines = &program->lines,
             lines = sv_vec_with_cap((end->row - start->row) + 1);

    if (start->row >= p_lines->size || end->row >= p_lines->size)
        terminate("[point_position_range] start and end rows are out of bounds",
                  EXIT_FAILURE);

    vec_insert_vec(&lines, p_lines, 0, start->row - 1,
                   (end->row - start->row) + 1);

    int32_t tab_size = 4;

    string_t formatted = string_from("");

    for (size_t i = 0; i < lines.size; i++)
    {
        string_view_t *sv = sv_vec_at(&lines, i);

        clean(string_free) string_t f_line,
            line = string_from_prt(sv->data, sv->len), l_no;

        string_init(&f_line, line.len ? 256 : 64);
        string_init(&l_no, 24);

        string_format(&l_no, "%zu", start->row + i);

        size_t l_start = i == 0 ? start->col - 1 : 0,
               l_end = i == lines.size - 1 ? end->col - 1
                                           : line.len - (line.len != 0);

        const char *color = color_of(severity);

        if (line.len)
        {

            string_insert(&line, S_RESET, l_end + 1);
            string_insert(&line, color, l_start);
        }

        clean(string_free) string_t emphasis =
            string_with_char('^', (l_end - l_start) + 1);

        string_format(&f_line,
                      "  %s%s  %s|%s   %s%s\n  %*s  %s|%s   %*s%s%s%s\n",
                      C_CYAN, l_no.data, C_BRIGHT_BLACK, S_RESET, line.data,
                      S_RESET, l_no.len, "", C_BRIGHT_BLACK, S_RESET, l_start,
                      "", color, emphasis.data, S_RESET);

        string_push_str(&formatted, &f_line);
    }

    //     // Apply color to the specific range
    //     auto l_start = i == start.row - 1 ? start.col - 1 : 0,
    //          l_end = (i == end.row - 1 ? end.col : line.size()) - 1;

    //     line.insert(l_end + 1, utils::Styles::Reset);
    //     line.insert(l_start, color);

    //     formatted += line + "\n";

    //     formatted += utils::tab(prefix_len + l_start, 1) + color +
    //                  std::string((l_end - l_start) + 1, '^') + "\n";
    // }

    return formatted;
}
reporter_t *reporter()
{
    static reporter_t CLI_Reporter;
    if (!initialized)
        CLI_Reporter = (reporter_t){
            .diagnostics = diag_vec_with_cap(8),
            .fmt = (diag_fmt)cli_fmt,
            .report = (reporter_fn)cli_report,
        };

    return &CLI_Reporter;
}

string_t cli_fmt(diagnostic_t *diagnostic)
{
    diagnostic_severity_t severity = diagnostic->severity;
    position_range_t *range = &diagnostic->range;

    position_t *start = &range->start, *end = &range->end;
    program_t *program = start->program;

    clean(string_free) string_t p_path = string_from(program->path),
                                current = string_with(getcwd(NULL, 0));

    if (current.data == NULL)
        terminate("[cli_fmt] failed to retrieve the current working directory",
                  EXIT_FAILURE);

    clean(string_free) string_t dir = string_substr(&current, 0, current.len);

    dirname(dir.data);
    dir.len = strlen(dir.data);

    clean(string_free)
        string_t cwp = string_substr(&current, dir.len,
                                     (current.len - dir.len) + 1),
                 location = string_substr(
                     &p_path, string_find(&p_path, cwp.data) + cwp.len,
                     p_path.len);

    string_t formatted,
        pointed clean(string_free) = point_position_range(severity, range);
    string_init(&formatted, 1024);

    sprintf(formatted.data, "%s%s:%zu:%zu: %s%s<%s%u> %s%s\n%s\n",
            C_BRIGHT_BLACK, location.data, start->row, start->col,
            color_of(severity), severity_to_string(severity),
            severity_prefix(severity), diagnostic->code, S_RESET,
            diagnostic->message.data, pointed.data);

    return formatted;
}

void cli_report(reporter_t *self)
{
    for (size_t i = 0; i < self->diagnostics.size; i++)
    {
        diagnostic_t *diagnostic = diag_vec_at(&self->diagnostics, i);
        clean(string_free) string_t fmt = self->fmt(diagnostic);

        printf("%s", fmt.data);
    }

    vec_free(&self->diagnostics);
}
