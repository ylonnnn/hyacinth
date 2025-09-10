#ifndef __UTILS_TYPES_RESULT_H__
#define __UTILS_TYPES_RESULT_H__

#define RESULT(OkTy, ErrTy, name)                                              \
    typedef struct name##_result                                               \
    {                                                                          \
        bool errored;                                                          \
        union name##_result_data                                               \
        {                                                                      \
            OkTy ok;                                                           \
            ErrTy err;                                                         \
        } data;                                                                \
    } name##_result_t;

// static inline void name##_result_ok(name##_result_t *result, OkTy val)     \
// {                                                                          \
// }                                                                          \
// result->data.ok = val;                                                 \
//                                                                            \
// static inline void name##_result_err(name##_result_t *result, ErrTy err)   \
// {                                                                          \
//     result->errored = true;                                                \
//     result->data.err = err;                                                \
// }

#define RESULT_OK(result, val)                                                 \
    result->errored = false;                                                   \
    result->data.ok = val;

#define RESULT_ERR(result, err)                                                \
    result->errored = true;                                                    \
    result->data.err = err;

#define RESULT_NONE                                                            \
    struct                                                                     \
    {                                                                          \
    }

#endif
