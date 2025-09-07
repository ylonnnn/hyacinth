#ifndef __CORE_PROGRAM_PROGRAM_H__
#define __CORE_PROGRAM_PROGRAM_H__

#include "utils/types/string.h"

#define FILE_EXTENSION ".hyc"

typedef struct program
{
    const char *path;
    string_t source;
    vector_t lines;
} program_t;

// Constructors
program_t program_from(const char *path);

// Destructor
void program_free(program_t *program);

// Helper

void program_read_source(program_t *program);

void program_execute(program_t *program);

#endif
