#include <assert.h>

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

void diagnostic_free(diagnostic_t *diagnostic)
{
    vec_free(&diagnostic->details);

    string_free(&diagnostic->message);
}

void diagnostic_move(diagnostic_t *dest, diagnostic_t *src)
{
    dest->severity = src->severity;
    dest->code = src->code;
    string_move(&dest->message, &src->message);
    dest->range = src->range;
    vec_move(&dest->details, &src->details);
}
