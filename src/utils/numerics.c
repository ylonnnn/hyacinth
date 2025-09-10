#include <ctype.h>

#include "utils/numerics.h"

bool is_digit_of(char c, uint32_t base)
{
    switch (base)
    {
        case 2:
            return c == '0' || c == '1';
        case 8:
            return c >= '0' && c <= '7';
        case 10:
            return isdigit(c);
        case 16:
            return isdigit(c) || (c >= 'a' && c <= 'f') ||
                   (c >= 'A' && c <= 'F');
    }

    return false;
}
