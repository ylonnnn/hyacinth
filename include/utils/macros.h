#ifndef __UTILS__MACROS_H__
#define __UTILS__MACROS_H__

/**
 * Automatically cleans up the object. The `cleanup` function provided acts
 * similarly to C++ destructors.
 */
#define clean(fn) __attribute__((cleanup(fn)))

#define TODO(message) printf("[TODO] %s\n", message);

#define MAX(a, b) (a > b ? a : b)
#define MIN(a, b) (a < b ? a : b)

#endif
