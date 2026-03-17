// Xin Runtime - C implementation of built-in functions
// This file is linked with compiled Xin code to provide runtime support

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <stdarg.h>

// Integer print
void xin_print_int(long long n) {
    printf("%lld", n);
}

// Float print
void xin_print_float(double n) {
    printf("%g", n);
}

// Boolean print
void xin_print_bool(int b) {
    printf("%s", b ? "true" : "false");
}

// String print
void xin_print_str(const char* s) {
    printf("%s", s);
}

// Newline
void xin_println() {
    printf("\n");
}

// String concatenation: string + string
char* xin_str_concat_ss(const char* a, const char* b) {
    // Handle NULL inputs - treat as empty strings
    if (!a) a = "";
    if (!b) b = "";

    size_t len_a = strlen(a);
    size_t len_b = strlen(b);
    char* result = (char*)malloc(len_a + len_b + 1);
    if (result) {
        strcpy(result, a);
        strcat(result, b);
    }
    return result;
}

// String concatenation: string + int
char* xin_str_concat_si(const char* a, long long b) {
    char buf[32];
    snprintf(buf, sizeof(buf), "%lld", b);
    return xin_str_concat_ss(a, buf);
}

// String concatenation: int + string
char* xin_str_concat_is(long long a, const char* b) {
    char buf[32];
    snprintf(buf, sizeof(buf), "%lld", a);
    return xin_str_concat_ss(buf, b);
}

// String concatenation: string + float
char* xin_str_concat_sf(const char* a, double b) {
    char buf[64];
    if (isnan(b)) {
        snprintf(buf, sizeof(buf), "NaN");
    } else if (isinf(b)) {
        snprintf(buf, sizeof(buf), b > 0 ? "Infinity" : "-Infinity");
    } else {
        snprintf(buf, sizeof(buf), "%g", b);
    }
    return xin_str_concat_ss(a, buf);
}

// String concatenation: float + string
char* xin_str_concat_fs(double a, const char* b) {
    char buf[64];
    if (isnan(a)) {
        snprintf(buf, sizeof(buf), "NaN");
    } else if (isinf(a)) {
        snprintf(buf, sizeof(buf), a > 0 ? "Infinity" : "-Infinity");
    } else {
        snprintf(buf, sizeof(buf), "%g", a);
    }
    return xin_str_concat_ss(buf, b);
}

// String concatenation: string + bool
char* xin_str_concat_sb(const char* a, int b) {
    return xin_str_concat_ss(a, b ? "true" : "false");
}

// String concatenation: bool + string
char* xin_str_concat_bs(int a, const char* b) {
    return xin_str_concat_ss(a ? "true" : "false", b);
}

// String deallocation
void xin_str_free(char* s) {
    if (s) {
        free(s);
    }
}

// Printf implementation with %b support for boolean
void xin_printf(const char* format, ...) {
    va_list args;
    va_start(args, format);

    const char* p = format;
    while (*p) {
        if (*p == '%' && *(p + 1)) {
            p++;
            // Parse width modifier
            int width = 0;
            while (*p >= '0' && *p <= '9') {
                width = width * 10 + (*p - '0');
                p++;
            }

            switch (*p) {
                case 'b': {
                    // Boolean support
                    int val = va_arg(args, int);
                    const char* str = val ? "true" : "false";
                    int len = val ? 4 : 5; // "true" or "false"
                    if (width > len) {
                        for (int i = 0; i < width - len; i++) {
                            putchar(' ');
                        }
                    }
                    printf("%s", str);
                    break;
                }
                case '%':
                    putchar('%');
                    break;
                case 'd':
                case 'i': {
                    long long val = va_arg(args, long long);
                    printf("%lld", val);
                    break;
                }
                case 'x': {
                    long long val = va_arg(args, long long);
                    printf("%llx", val);
                    break;
                }
                case 'X': {
                    long long val = va_arg(args, long long);
                    printf("%llX", val);
                    break;
                }
                case 'o': {
                    long long val = va_arg(args, long long);
                    printf("%llo", val);
                    break;
                }
                case 'c': {
                    int val = va_arg(args, int);
                    putchar(val);
                    break;
                }
                case 'f': {
                    double val = va_arg(args, double);
                    printf("%g", val);
                    break;
                }
                case 's': {
                    const char* val = va_arg(args, const char*);
                    printf("%s", val ? val : "(null)");
                    break;
                }
                default:
                    putchar(*p);
            }
            p++;
        } else {
            putchar(*p);
            p++;
        }
    }

    va_end(args);
}

// Printf wrapper functions for different argument counts
// These are needed because Cranelift doesn't support variadic functions directly

// 1 argument (format string only)
void xin_printf_1(const char* fmt) {
    xin_printf(fmt);
}

// 2 arguments with different types
void xin_printf_2_i(const char* fmt, long long a1) {
    xin_printf(fmt, a1);
}

void xin_printf_2_f(const char* fmt, double a1) {
    xin_printf(fmt, a1);
}

void xin_printf_2_s(const char* fmt, const char* a1) {
    xin_printf(fmt, a1);
}

// 3 arguments
void xin_printf_3_ii(const char* fmt, long long a1, long long a2) {
    xin_printf(fmt, a1, a2);
}

void xin_printf_3_si(const char* fmt, const char* a1, long long a2) {
    xin_printf(fmt, a1, a2);
}

void xin_printf_3_sf(const char* fmt, const char* a1, double a2) {
    xin_printf(fmt, a1, a2);
}

void xin_printf_3_ss(const char* fmt, const char* a1, const char* a2) {
    xin_printf(fmt, a1, a2);
}