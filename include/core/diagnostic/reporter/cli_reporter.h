#ifndef __CORE_DIAGNOSTIC_REPORTER_CLI_REPORTER_H__
#define __CORE_DIAGNOSTIC_REPORTER_CLI_REPORTER_H__

#include "core/diagnostic/diagnostic.h"
#include "core/diagnostic/reporter/reporter.h"
#include "utils/style.h"

reporter_t *reporter();

static const text_style INFO_COLOR = C_BLUE, WARN_COLOR = C_YELLOW,
                        ERROR_COLOR = C_RED;

string_t cli_fmt(diagnostic_t *diagnostic);
report_result_t cli_report(reporter_t *reporter);

#endif
