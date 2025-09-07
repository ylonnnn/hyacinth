#ifndef __CORE_DIAGNOSTIC_REPORTER_REPORTER_H__
#define __CORE_DIAGNOSTIC_REPORTER_REPORTER_H__

#include "core/diagnostic/diagnostic.h"
#include "utils/types/string.h"

typedef struct reporter reporter_t;

typedef void (*reporter_fn)(reporter_t *self);
typedef string_t (*diag_fmt)(diagnostic_t *diagnostic);

typedef struct reporter
{
    diag_fmt fmt;
    reporter_fn report;

    vector_t diagnostics;
} reporter_t;

#endif
