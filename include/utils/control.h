#ifndef __UTILS_CONTROL_H__
#define __UTILS_CONTROL_H__

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

inline static void terminate(const char *message, uint8_t code)
{
    printf("%s\n", message);
    exit(code);
}

#endif
