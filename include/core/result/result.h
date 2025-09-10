#ifndef __CORE_RESULT_RESULT_H__
#define __CORE_RESULT_RESULT_H__

#include "utils/types/vector.h"

typedef enum result_status
{
    SUCCESS,
    FAIL,
} result_status_t;

typedef void (*result_data_destr)(void *self);

typedef struct result_data_opts
{
    result_data_destr destr;
} result_data_opts_t;

typedef struct result_data
{
    void *data;
    result_data_opts_t opts;
} result_data_t;

// Constructors
result_data_t result_data_from(void *data, result_data_opts_t opts);

// Destructors
void result_data_free(result_data_t *data);

#define RES_DATA(T, data, val) *(T *)data.data = val;

typedef struct result
{
    result_status_t status;
    result_data_t data;
    vector_t diagnostics;
} result_t;

// Constructors
result_t result_create(result_status_t status);

// Destructors
void result_free(result_t *result);

#define RESULT_SET(result, status, data, diagnostics)                          \
    result.status = status, result.data = data,                                \
    result.diagnostics = diagnostics;

// Helper

/**
 * Adapts the provided `status` and `diagnostics` to the result.
 *
 * NOTE: `diagnostics` will be moved to the provided result
 */
void result_adapt(result_t *result, result_status_t status,
                  vector_t *diagnostics);

#endif
