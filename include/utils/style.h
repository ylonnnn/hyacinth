#ifndef __UTILS_STYLE_H__
#define __UTILS_STYLE_H__

typedef const char *text_style;

// Color
#define C_BLACK "\033[30m"
#define C_RED "\033[31m"
#define C_GREEN "\033[32m"
#define C_YELLOW "\033[33m"
#define C_BLUE "\033[34m"
#define C_MAGENTA "\033[35m"
#define C_CYAN "\033[36m"
#define C_WHITE "\033[37m"

#define C_BRIGHT_BLACK "\033[90m"
#define C_BRIGHT_RED "\033[91m"
#define C_BRIGHT_GREEN "\033[92m"
#define C_BRIGHT_YELLOW "\033[93m"
#define C_BRIGHT_BLUE "\033[94m"
#define C_BRIGHT_MAGENTA "\033[95m"
#define C_BRIGHT_CYAN "\033[96m"
#define C_BRIGHT_WHITE "\033[97m"

#define C_BACKGROUND_RED "\033[41m"
#define C_BACKGROUND_GREEN "\033[42m"
#define C_BACKGROUND_YELLOW "\033[43m"
#define C_BACKGROUND_BLUE "\033[44m"
#define C_BACKGROUND_MAGENTA "\033[45m"
#define C_BACKGROUND_CYAN "\033[46m"
#define C_BACKGROUND_WHITE "\033[47m"

#define C_BACKGROUND_BRIGHT_BLACK "\033[100m"
#define C_BACKGROUND_BRIGHT_RED "\033[101m"
#define C_BACKGROUND_BRIGHT_GREEN "\033[102m"
#define C_BACKGROUND_BRIGHT_YELLOW "\033[103m"
#define C_BACKGROUND_BRIGHT_BLUE "\033[104m"
#define C_BACKGROUND_BRIGHT_MAGENTA "\033[105m"
#define C_BACKGROUND_BRIGHT_CYAN "\033[106m"
#define C_BACKGROUND_BRIGHT_WHITE "\033[107m"

// Style
#define S_RESET "\033[0m"
#define S_DIM "\033[2m"
#define S_UNDERLINE "\033[4m"
#define S_BLINK "\033[5m"
#define S_REVERSE "\033[7m"
#define S_HIDDEN "\033[8m"

#endif
