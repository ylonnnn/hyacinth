#ifndef __UTILS_TIME_H__
#define __UTILS_TIME_H__

#include <stdint.h>

#define NANOSECONDS_PER_MILLISECOND 1000000

#ifdef _WIN32
#include <stdbool.h>
#include <windows.h>

static inline double get_nanosec()
{
    static LARGE_INTEGER frequency;
    static bool initialized = false;

    if (!initialized)
    {
        QueryPerformanceFrequency(&frequency);
        initialized = true;
    }

    LARGE_INTEGER counter;
    QueryPerformanceCounter(&counter);

    return (double)counter.QuadPart * 1e9 / (double)frequency.QuadPart;
}

#else
#include <bits/time.h>
#include <time.h>

static inline double get_nanosec()
{
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);

    return (double)ts.tv_nsec;
}

#endif

#endif
