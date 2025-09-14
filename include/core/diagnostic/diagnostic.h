#ifndef __CORE_DIAGNOSTIC_DIAGNOSTIC_H__
#define __CORE_DIAGNOSTIC_DIAGNOSTIC_H__

#include <stdint.h>

#include "core/position/position.h"
#include "utils/types/string.h"
#include "utils/types/vector.h"

typedef enum diagnostic_severity
{
    DIAG_INFO,
    DIAG_WARN,
    DIAG_ERROR
} diagnostic_severity_t;

typedef struct diagnostic
{
    diagnostic_severity_t severity;
    uint32_t code;
    string_t message;
    vector_t details;
    position_range_t range;
} diagnostic_t;

// Constructors
void diagnostic_init(diagnostic_t *diagnostic);
diagnostic_t diagnostic_from(diagnostic_severity_t severity, uint32_t code,
                             string_t message, position_range_t range);

diagnostic_t *diagnostic_create_on(diagnostic_t *addr,
                                   diagnostic_severity_t severity,
                                   uint32_t code, string_t message,
                                   position_range_t range);

diagnostic_t diagnostic_copy(diagnostic_t *diag);
void diagnostic_copy_to(diagnostic_t *diag, diagnostic_t *dest);
void diagnostic_move(diagnostic_t *dest, diagnostic_t *src);

// Destructors
void diagnostic_free(diagnostic_t *diagnostic);

// Helper

T_VEC_CONSTR(diagnostic_t, diag_vec,
             ((vec_opts_t){(vec_el_destr)diagnostic_free,
                           (vec_el_cp)diagnostic_copy_to,
                           (vec_el_mv)diagnostic_move}))
T_VEC_AT(diagnostic_t, diag_vec)
T_VEC_PUSH(diagnostic_t, diag_vec_push)
T_VEC_RESET(diagnostic_t, diag_vec_reset)
T_VEC_USE(diagnostic_t, diag_vec_use)

#endif
