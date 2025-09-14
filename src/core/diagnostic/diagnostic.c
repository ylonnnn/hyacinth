#include <assert.h>
#include <stdio.h>

#include "core/diagnostic/diagnostic.h"
#include "utils/types/string.h"
#include "utils/types/vector.h"

void diagnostic_init(diagnostic_t *diagnostic)
{
    assert(diagnostic != NULL);
    diagnostic->details = diag_vec_with_cap(8);
}

diagnostic_t diagnostic_from(diagnostic_severity_t severity, uint32_t code,
                             string_t message, position_range_t range)
{
    return (diagnostic_t){
        .severity = severity,
        .code = code,
        .message = message,
        .details = diag_vec_with_cap(8),
        .range = range,
    };
}

diagnostic_t *diagnostic_create_on(diagnostic_t *addr,
                                   diagnostic_severity_t severity,
                                   uint32_t code, string_t message,
                                   position_range_t range)
{
    assert(addr != NULL);

    *addr = diagnostic_from(severity, code, message, range);

    return addr;
}

diagnostic_t diagnostic_copy(diagnostic_t *diag)
{
    assert(diag != NULL);

    diagnostic_t copy = diagnostic_from(
        diag->severity, diag->code, string_copy(&diag->message), diag->range);

    vec_copy_to(&diag->details, &copy.details);

    return copy;
}

void diagnostic_copy_to(diagnostic_t *diag, diagnostic_t *dest)
{
    assert(diag != NULL);
    assert(dest != NULL);

    *dest = diagnostic_copy(diag);
}

void diagnostic_free(diagnostic_t *diagnostic)
{
    string_free(&diagnostic->message);
    vec_free(&diagnostic->details);
}

void diagnostic_move(diagnostic_t *dest, diagnostic_t *src)
{
    dest->severity = src->severity;
    dest->code = src->code;
    string_move(&dest->message, &src->message);
    dest->range = src->range;
    vec_move(&dest->details, &src->details);
}
