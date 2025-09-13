#include "core/result/result.h"
#include "core/diagnostic/diagnostic.h"

result_data_t result_data_from(void *data, result_data_opts_t opts)
{
    return (result_data_t){
        .data = data,
        .opts = opts,
    };
}

void result_data_free(result_data_t *data)
{
    if (data->opts.destr)
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

void result_free(result_t *result)
{
    result_data_free(&result->data);
    vec_free(&result->diagnostics);
}

void result_adapt(result_t *result, result_status_t status,
                  vector_t *diagnostics)
{
    result->status = status;
    vec_move(&result->diagnostics, diagnostics);
}

void result_info(result_t *result, diagnostic_t *diagnostic)
{
    diagnostic_move(diag_vec_use(&result->diagnostics), diagnostic);
}

void result_warn(result_t *result, diagnostic_t *diagnostic)
{
    diagnostic_move(diag_vec_use(&result->diagnostics), diagnostic);
}

void result_error(result_t *result, diagnostic_t *diagnostic)
{
    result->status = RES_FAIL;
    diagnostic_move(diag_vec_use(&result->diagnostics), diagnostic);
}
