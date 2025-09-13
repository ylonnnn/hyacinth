#ifndef __UTILS_TIME_H__
#define __UTILS_TIME_H__

#include <bits/time.h>
#include <time.h>

const long NANOSECONDS_PER_MILLISECOND = 1000000;

static inline double get_nanosec()
{
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);

    return (double)ts.tv_nsec;
}

#endif
