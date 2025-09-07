#ifndef __CORE_POSITION_POSITION_H__
#define __CORE_POSITION_POSITION_H__

#include "core/program/program.h"

typedef struct position
{
    program_t *program;
    size_t row, col, offset;
} position_t;

typedef struct position_range
{
    position_t start, end;
} position_range_t;

#endif
