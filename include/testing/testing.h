#ifndef __TESTING_TESTING_H__
#define __TESTING_TESTING_H__

#define __testing

#include "utils/types/vector.h"

vector_t files_from(const char *dirpath);

void begin_tests(const char *testpath);

#endif
