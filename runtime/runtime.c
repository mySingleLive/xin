// Xin Runtime - C implementation of built-in functions
// This file is linked with compiled Xin code to provide runtime support

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <stdarg.h>
#include <stdint.h>

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

// Convert integer to string
char* xin_int_to_str(int64_t n) {
    char* buf = malloc(32);
    if (buf == NULL) return NULL;
    snprintf(buf, 32, "%lld", (long long)n);
    return buf;
}

// Convert float to string
char* xin_float_to_str(double d) {
    char* buf = malloc(64);
    if (buf == NULL) return NULL;
    snprintf(buf, 64, "%g", d);
    return buf;
}

// Convert boolean to string
char* xin_bool_to_str(int8_t b) {
    const char* val = b ? "true" : "false";
    char* buf = malloc(8);
    if (buf == NULL) return NULL;
    strcpy(buf, val);
    return buf;
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

// ========== Array Runtime ==========

// 数组结构
typedef struct {
    void** data;        // 元素指针数组
    int64_t length;      // 当前长度
    int64_t capacity;    // 容量
} xin_array;

// 创建数组
xin_array* xin_array_new(int64_t capacity) {
    xin_array* arr = (xin_array*)malloc(sizeof(xin_array));
    if (!arr) return NULL;

    arr->data = (void**)calloc(capacity > 0 ? capacity : 4, sizeof(void*));
    if (!arr->data) {
        free(arr);
        return NULL;
    }

    arr->length = 0;
    arr->capacity = capacity > 0 ? capacity : 4;
    return arr;
}

// 获取元素（越界 panic）
void* xin_array_get(xin_array* arr, int64_t index) {
    if (index < 0 || index >= arr->length) {
        fprintf(stderr, "ArrayIndexOutOfBoundsError: index %lld out of bounds for length %lld\n",
                (long long)index, (long long)arr->length);
        exit(1);
    }
    return arr->data[index];
}

// 设置元素（用于初始化，可设置 capacity 范围内的索引）
void xin_array_set(xin_array* arr, int64_t index, void* value) {
    if (index < 0 || index >= arr->capacity) {
        fprintf(stderr, "ArrayIndexOutOfBoundsError: index %lld out of bounds for capacity %lld\n",
                (long long)index, (long long)arr->capacity);
        exit(1);
    }
    arr->data[index] = value;
    // 更新长度以包含设置的索引
    if (index >= arr->length) {
        arr->length = index + 1;
    }
}

// 追加元素
void xin_array_push(xin_array* arr, void* value) {
    if (arr->length >= arr->capacity) {
        // 扩容
        int64_t new_capacity = arr->capacity * 2;
        void** new_data = (void**)realloc(arr->data, new_capacity * sizeof(void*));
        if (!new_data) {
            fprintf(stderr, "MemoryError: failed to expand array\n");
            exit(1);
        }
        arr->data = new_data;
        arr->capacity = new_capacity;
    }
    arr->data[arr->length++] = value;
}

// 弹出元素
void* xin_array_pop(xin_array* arr) {
    if (arr->length == 0) {
        fprintf(stderr, "ArrayPopError: cannot pop from empty array\n");
        exit(1);
    }
    return arr->data[--arr->length];
}

// 获取长度
int64_t xin_array_len(xin_array* arr) {
    return arr->length;
}

// ========== Map Runtime ==========

// Map 条目结构
typedef struct xin_map_entry {
    char* key;                  // 键（字符串）
    void* value;                // 值（通用指针）
    struct xin_map_entry* next; // 链表下一个节点（用于哈希冲突）
} xin_map_entry;

// Map 结构（使用简单的哈希表实现）
typedef struct {
    xin_map_entry** buckets;    // 哈希桶数组
    int64_t bucket_count;       // 桶数量
    int64_t size;               // 键值对数量
} xin_map;

// 简单的字符串哈希函数（djb2）
static uint64_t map_hash(const char* str) {
    uint64_t hash = 5381;
    int c;
    while ((c = *str++)) {
        hash = ((hash << 5) + hash) + c;  // hash * 33 + c
    }
    return hash;
}

// 创建 Map
xin_map* xin_map_new() {
    xin_map* map = (xin_map*)malloc(sizeof(xin_map));
    if (!map) return NULL;

    // 初始 16 个桶
    int64_t initial_buckets = 16;
    map->buckets = (xin_map_entry**)calloc(initial_buckets, sizeof(xin_map_entry*));
    if (!map->buckets) {
        free(map);
        return NULL;
    }

    map->bucket_count = initial_buckets;
    map->size = 0;
    return map;
}

// 设置键值对
void xin_map_set(xin_map* map, const char* key, void* value) {
    uint64_t hash = map_hash(key);
    int64_t index = hash % map->bucket_count;

    // 查找是否已存在该键
    xin_map_entry* entry = map->buckets[index];
    while (entry) {
        if (strcmp(entry->key, key) == 0) {
            // 键已存在，更新值
            entry->value = value;
            return;
        }
        entry = entry->next;
    }

    // 创建新条目
    xin_map_entry* new_entry = (xin_map_entry*)malloc(sizeof(xin_map_entry));
    if (!new_entry) {
        fprintf(stderr, "MemoryError: failed to allocate map entry\n");
        exit(1);
    }

    // 复制键字符串
    new_entry->key = strdup(key);
    if (!new_entry->key) {
        free(new_entry);
        fprintf(stderr, "MemoryError: failed to duplicate key\n");
        exit(1);
    }

    new_entry->value = value;
    new_entry->next = map->buckets[index];
    map->buckets[index] = new_entry;
    map->size++;
}

// 获取值
void* xin_map_get(xin_map* map, const char* key) {
    uint64_t hash = map_hash(key);
    int64_t index = hash % map->bucket_count;

    xin_map_entry* entry = map->buckets[index];
    while (entry) {
        if (strcmp(entry->key, key) == 0) {
            return entry->value;
        }
        entry = entry->next;
    }

    return NULL;  // 键不存在
}

// 获取键值对数量
int64_t xin_map_len(xin_map* map) {
    return map->size;
}

// 检查键是否存在
int xin_map_has(xin_map* map, const char* key) {
    uint64_t hash = map_hash(key);
    int64_t index = hash % map->bucket_count;

    xin_map_entry* entry = map->buckets[index];
    while (entry) {
        if (strcmp(entry->key, key) == 0) {
            return 1;  // 存在
        }
        entry = entry->next;
    }

    return 0;  // 不存在
}

// 删除键值对
int xin_map_remove(xin_map* map, const char* key) {
    uint64_t hash = map_hash(key);
    int64_t index = hash % map->bucket_count;

    xin_map_entry* entry = map->buckets[index];
    xin_map_entry* prev = NULL;

    while (entry) {
        if (strcmp(entry->key, key) == 0) {
            // 找到了，删除
            if (prev) {
                prev->next = entry->next;
            } else {
                map->buckets[index] = entry->next;
            }

            free(entry->key);
            free(entry);
            map->size--;
            return 1;  // 删除成功
        }
        prev = entry;
        entry = entry->next;
    }

    return 0;  // 键不存在
}

// 获取所有键（返回数组）
xin_array* xin_map_keys(xin_map* map) {
    xin_array* keys = xin_array_new(map->size);
    if (!keys) return NULL;

    for (int64_t i = 0; i < map->bucket_count; i++) {
        xin_map_entry* entry = map->buckets[i];
        while (entry) {
            // 复制键字符串作为数组元素
            char* key_copy = strdup(entry->key);
            if (!key_copy) {
                fprintf(stderr, "MemoryError: failed to duplicate key\n");
                exit(1);
            }
            xin_array_push(keys, key_copy);
            entry = entry->next;
        }
    }

    return keys;
}

// 获取所有值（返回数组）
xin_array* xin_map_values(xin_map* map) {
    xin_array* values = xin_array_new(map->size);
    if (!values) return NULL;

    for (int64_t i = 0; i < map->bucket_count; i++) {
        xin_map_entry* entry = map->buckets[i];
        while (entry) {
            xin_array_push(values, entry->value);
            entry = entry->next;
        }
    }

    return values;
}
