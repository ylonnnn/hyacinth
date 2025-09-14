#include <assert.h>
#include <stdio.h>

#include "core/diagnostic/diagnostic.h"
#include "core/result/result.h"
#include "utils/types/vector.h"

result_data_t result_data_from(void *data, result_data_opts_t opts)
{
    return (result_data_t){
        .data = data,
        .opts = opts,
    };
}

void result_data_move(result_data_t *dest, result_data_t *src)
{
    if (src->data)
    {
        src->opts.mv(src->data, dest->data);
        src->data = NULL;
    }

    dest->opts = src->opts;
}

void result_data_free(result_data_t *data)
{
    if (data->data != NULL && data->opts.destr)
        data->opts.destr(data->data);

    data->data = NULL;
}

result_t result_create(result_status_t status)
{
    return (result_t){.status = status,
                      .data =
                          result_data_from(NULL, (result_data_opts_t){NULL}),
                      .diagnostics = diag_vec_with_cap(8)};
}

void result_move(result_t *dest, result_t *src)
{
    assert(dest != NULL && src != NULL);

    dest->status = src->status;
    vec_move(&dest->diagnostics, &src->diagnostics);
    result_data_move(&dest->data, &src->data);
}

void result_free(result_t *result)
{
    assert(result != NULL);

    result_data_free(&result->data);
    vec_free(&result->diagnostics);
}

void result_adapt(result_t *result, result_status_t status,
                  vector_t *diagnostics)
{
    assert(result != NULL && diagnostics != NULL);

    result->status = status;
    vec_insert_full_vec(&result->diagnostics, diagnostics,
                        result->diagnostics.size, true);
}

void result_info(result_t *result, diagnostic_t *diagnostic)
{
    assert(result != NULL);
    diagnostic_move(diag_vec_use(&result->diagnostics), diagnostic);
}

void result_warn(result_t *result, diagnostic_t *diagnostic)
{
    assert(result != NULL);
    diagnostic_move(diag_vec_use(&result->diagnostics), diagnostic);
}

void result_error(result_t *result, diagnostic_t *diagnostic)
{
    assert(result != NULL);

    result->status = RES_FAIL;
    diagnostic_move(diag_vec_use(&result->diagnostics), diagnostic);
}
