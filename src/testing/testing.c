#include <string.h>
#include <sys/stat.h>

#include "core/program/program.h"
#include "testing/testing.h"
#include "utils/control.h"
#include "utils/macros.h"
#include "utils/style.h"
#include "utils/types/string.h"
#include "utils/types/vector.h"

#ifdef _WIN32
#include <tchar.h>
#include <windows.h>
#define PATH_MAX MAX_PATH
#else
#include <dirent.h>
#include <unistd.h>
#endif

vector_t files_from(const char *dirpath)
{
    vector_t path_vec = str_vec_with_cap(8);

#ifdef _WIN32
    WIN32_FIND_DATA ff_data;
    char spath[MAX_PATH];

    snprintf(spath, MAX_PATH, "%s\\*", dirpath);

    HANDLE h_find = FindFirstFile(spath, &ff_data);
    if (h_find == INVALID_HANDLE_VALUE)
        terminate("[files_from] failed to open directory", EXIT_FAILURE);

    do
    {
        if (strcmp(ff_data.cFileName, ".") == 0 ||
            strcmp(ff_data.cFileName, "..") == 0)
            continue;

        char cpath[PATH_MAX];
        snprintf(cpath, PATH_MAX, "%s\\%s", dirpath, ff_data.cFileName);

        struct stat buf;
        int status = stat(cpath, &buf);

        if (S_ISDIR(buf.st_mode))
        {
            clean(vec_free) vector_t files = files_from(cpath);
            vec_insert_full_vec(&path_vec, &files, path_vec.size);

            continue;
        }

        *str_vec_use(&path_vec) = string_from(cpath);

    } while (FindNextFile(h_find, &ff_data) != 0);

    FindClose(h_find);

#else
    DIR *dir;
    struct dirent *entry;

    dir = opendir(dirpath);
    if (dir == NULL)
        terminate("[files_from] failed to open directory", EXIT_FAILURE);

    while ((entry = readdir(dir)) != NULL)
    {
        if (strcmp(entry->d_name, ".") == 0 || strcmp(entry->d_name, "..") == 0)
            continue;

        char cpath[PATH_MAX];
        snprintf(cpath, PATH_MAX, "%s/%s", dirpath, entry->d_name);

        struct stat buf;
        int status = stat(cpath, &buf);

        if (S_ISDIR(buf.st_mode))
        {
            // Does not require automatic clean-up as the full vector is moved
            vector_t files = files_from(cpath);
            vec_insert_full_vec(&path_vec, &files, path_vec.size, true);

            continue;
        }

        *str_vec_use(&path_vec) = string_from(cpath);
    }

    closedir(dir);

#endif

    return path_vec;
}

void begin_tests(const char *testpath)
{
    clean(vec_free) vector_t files = files_from(testpath);

    for (size_t i = 0; i < files.size; i++)
    {
        string_t *filepath = str_vec_at(&files, i);
        printf("%s[%sTEST%s] %sRunning test on: %s%s%s...%s\n", C_BRIGHT_BLACK,
               C_GREEN, C_BRIGHT_BLACK, C_WHITE, C_GREEN, filepath->data,
               C_BRIGHT_BLACK, S_RESET);

        clean(program_free) program_t program = program_from(filepath->data);
        program_execute(&program);

        printf("%s[%sTEST%s] %sRan test on: %s%s%s\n", C_BRIGHT_BLACK, C_GREEN,
               C_BRIGHT_BLACK, C_WHITE, C_GREEN, filepath->data, S_RESET);
    }
}
