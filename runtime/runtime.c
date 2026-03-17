// Xin Runtime - C implementation of built-in functions
// This file is linked with compiled Xin code to provide runtime support

#include <stdio.h>

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