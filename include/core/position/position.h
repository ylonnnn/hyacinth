#ifndef __CORE_POSITION_POSITION_H__
#define __CORE_POSITION_POSITION_H__

#include <stdint.h>

#include "core/program/program.h"

typedef struct position
{
    program_t *program;
    size_t offset;
    uint32_t row, col;
} position_t;

typedef struct position_range
{
    position_t start, end;
} position_range_t;

#endif
