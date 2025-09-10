#ifndef __CORE_PROGRAM_PROGRAM_H__
#define __CORE_PROGRAM_PROGRAM_H__

#include "utils/types/string.h"

#define FILE_EXT ".hyc"

typedef struct lexer lexer_t;

typedef struct program
{
    const char *path;
    string_t source;
    vector_t lines;

    lexer_t *lexer; // Heap-allocated due to incompleteness of type
                    // Including the completed type creates circular dependency
} program_t;

// Constructors
program_t program_from(const char *path);

// Destructor
void program_free(program_t *program);

// Helper

void program_initialize_lexer(program_t *program);

void program_read_source(program_t *program);

void program_execute(program_t *program);

#endif
