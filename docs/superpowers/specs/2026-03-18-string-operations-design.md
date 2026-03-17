# Xin 字符串操作与内置打印函数设计

**日期**: 2026-03-18

## 1. 概述

本文档描述 Xin 语言中字符串拼接操作和内置打印函数（`println`、`print`、`printf`）的设计规范。

## 2. 字符串拼接

### 2.1 语义规则

当 `+` 操作符的任意一侧是 `string` 类型时，触发字符串拼接语义：

```xin
let s1 = "Hello" + " World"        // "Hello World"
let s2 = "Count: " + 10            // "Count: 10"
let s3 = 3.14 + " is pi"           // "3.14 is pi"
let s4 = "Flag: " + true           // "Flag: true"
let s5 = "A" + "B" + "C"           // "ABC" (左结合)
```

### 2.2 类型转换规则

| 操作数类型 | 字符串表示 |
|-----------|-----------|
| `int` | 十进制字符串，如 `42` → `"42"`，`-7` → `"-7"` |
| `float` | 最精简表示，如 `3.14` → `"3.14"`，`100.0` → `"100"` |
| `float` (特殊值) | `NaN` → `"NaN"`，`Infinity` → `"Infinity"`，`-Infinity` → `"-Infinity"` |
| `bool` | `"true"` 或 `"false"` |

### 2.3 类型检查规则

在类型检查器中，`Binary Add` 操作符的处理逻辑：

```
if left_type == string || right_type == string:
    return string
else if left_type == int && right_type == int:
    return int
else if left_type == float || right_type == float:
    return float
else:
    error: type mismatch
```

**错误消息格式**:
```
error: incompatible types for '+' operator
  --> file.xin:5:9
   |
5  |     let x = someStruct + 10
   |             ^^^^^^^^^^^^^^^ cannot add 'SomeStruct' and 'int'
   |
help: strings can be concatenated with '+', or use explicit conversion
```

### 2.4 运行时函数

运行时需要提供以下字符串拼接函数：

| 函数 | 参数类型 | 说明 |
|-----|---------|------|
| `xin_str_concat_ss` | (str, str) | 字符串 + 字符串 |
| `xin_str_concat_si` | (str, int) | 字符串 + 整数 |
| `xin_str_concat_is` | (int, str) | 整数 + 字符串 |
| `xin_str_concat_sf` | (str, float) | 字符串 + 浮点数 |
| `xin_str_concat_fs` | (float, str) | 浮点数 + 字符串 |
| `xin_str_concat_sb` | (str, bool) | 字符串 + 布尔值 |
| `xin_str_concat_bs` | (bool, str) | 布尔值 + 字符串 |

### 2.5 内存管理

拼接产生的新字符串在堆上分配。

**MVP 阶段策略**:
- 运行时维护一个全局字符串分配列表
- 每次拼接时，新字符串添加到列表中
- 程序退出时，`xin_runtime_cleanup()` 函数释放所有分配的字符串

**运行时函数**:
```c
// 初始化运行时（程序启动时调用）
void xin_runtime_init(void);

// 清理所有分配的字符串（程序退出时调用）
void xin_runtime_cleanup(void);
```

**未来扩展**:
- 集成到编译期 GC 系统
- 实现引用计数或作用域追踪

## 3. 内置打印函数

### 3.1 println 函数

**签名**: `println(value: any) void`

**行为**:
- 接受任意类型的单个参数
- 根据参数类型调用对应的打印函数
- 打印后输出换行符
- 返回 `void`

**示例**:
```xin
println(42)           // 输出: 42\n
println(3.14)         // 输出: 3.14\n
println("hello")      // 输出: hello\n
println(true)         // 输出: true\n
```

**实现策略**:
- 语义分析阶段识别 `println` 调用
- IR 生成阶段根据参数类型选择运行时函数：
  - `int` → `xin_println_int`
  - `float` → `xin_println_float`
  - `string` → `xin_println_str`
  - `bool` → `xin_println_bool`

### 3.2 print 函数

**签名**: `print(value: any) void`

**行为**:
- 与 `println` 相同，但不输出换行符

**示例**:
```xin
print("Name: ")
print("Alice")
println("")           // 手动换行
// 输出: Name: Alice\n
```

**实现策略**:
- 运行时函数：
  - `xin_print_int(long long n)`
  - `xin_print_float(double n)`
  - `xin_print_str(const char* s)`
  - `xin_print_bool(int b)`

### 3.3 printf 函数

**签名**: `printf(format: string, args...) void`

**行为**:
- 第一个参数是格式字符串
- 支持占位符替换
- 返回 `void`

**支持的占位符**:

| 占位符 | 类型 | 说明 |
|-------|------|------|
| `%d`, `%ld` | int | 整数 |
| `%f`, `%lf` | float | 浮点数 |
| `%s` | string | 字符串 |
| `%b` | bool | 布尔值（Xin 扩展） |
| `%c` | int | 字符（ASCII 码） |
| `%x` | int | 十六进制（小写） |
| `%X` | int | 十六进制（大写） |
| `%o` | int | 八进制 |
| `%%` | - | 百分号字面量 |

**宽度与精度**:
- `%5d` - 最小宽度 5，右对齐
- `%-5d` - 最小宽度 5，左对齐
- `%.2f` - 保留 2 位小数
- `%8.2f` - 最小宽度 8，保留 2 位小数

**示例**:
```xin
printf("Name: %s, Age: %d\n", "Alice", 30)
// 输出: Name: Alice, Age: 30

printf("Price: $%.2f\n", 19.99)
// 输出: Price: $19.99

printf("Hex: 0x%X\n", 255)
// 输出: Hex: 0xFF
```

**错误处理**（编译期）:
- 占位符数量与参数数量不匹配 → 编译错误
- 参数类型与占位符类型不匹配 → 编译错误
- 未知占位符（如 `%z`）→ 编译错误

**错误处理**（运行时）:
- 格式字符串末尾的孤立 `%` → 输出 `%` 并继续
- 空指针字符串 → 输出 `(null)`

**实现策略**:
- IR 生成阶段传递格式字符串指针和参数列表
- 运行时直接调用 C 的 `vprintf`

### 3.4 类型检查

**println/print**:
- 参数数量必须为 1
- 参数可以是任意类型
- 返回类型为 `void`

**printf**:
- 第一个参数必须是 `string` 类型
- 参数数量必须与格式字符串中的占位符数量匹配（编译期错误）
- 参数类型必须与对应占位符兼容（编译期错误）
- 返回类型为 `void`

**占位符类型检查规则**:
- `%d`, `%ld`, `%x`, `%X`, `%o`, `%c` → 需要 `int` 类型
- `%f`, `%lf` → 需要 `float` 类型
- `%s` → 需要 `string` 类型
- `%b` → 需要 `bool` 类型

**占位符解析算法**:
1. 扫描格式字符串，识别所有 `%` 后跟的有效占位符
2. 统计占位符数量（跳过 `%%`）
3. 检查参数数量是否匹配
4. 按顺序检查每个参数类型是否与占位符匹配

**错误消息格式**:
```
error: printf argument count mismatch
  --> file.xin:3:5
   |
3  |     printf("%d %s\n", 42)
   |     ^^^^^^^^^^^^^^^^^^^^^ expected 2 arguments, found 1
```

```
error: printf argument type mismatch
  --> file.xin:3:5
   |
3  |     printf("%d\n", "hello")
   |                    ^^^^^^^ expected 'int' for '%d', found 'string'
```

## 4. 实现细节

### 4.1 修改文件列表

| 文件 | 状态 | 修改内容 |
|-----|------|---------|
| `crates/xin-semantic/src/type_check.rs` | 修改 | 字符串 `+` 类型检查；`printf` 类型检查 |
| `crates/xin-ir/src/ir.rs` | 修改 | 添加字符串拼接 IR 指令 |
| `crates/xin-ir/src/builder.rs` | 修改 | 字符串拼接 IR 生成 |
| `crates/xin-codegen/src/aot.rs` | 修改 | 字符串拼接代码生成 |
| `runtime/runtime.c` | 修改 | 字符串拼接运行时函数；`printf` 实现 |

### 4.2 IR 扩展

添加新的 IR 指令：

```rust
/// String concatenation
StringConcat {
    result: Value,
    left: Value,
    left_type: ConcatType,
    right: Value,
    right_type: ConcatType,
}

enum ConcatType {
    String,
    Int,
    Float,
    Bool,
}
```

### 4.3 运行时函数签名

```c
// String concatenation
char* xin_str_concat_ss(const char* a, const char* b);
char* xin_str_concat_si(const char* a, long long b);
char* xin_str_concat_is(long long a, const char* b);
char* xin_str_concat_sf(const char* a, double b);
char* xin_str_concat_fs(double a, const char* b);
char* xin_str_concat_sb(const char* a, int b);
char* xin_str_concat_bs(int a, const char* b);

// Print functions (existing)
void xin_print_int(long long n);
void xin_print_float(double n);
void xin_print_str(const char* s);
void xin_print_bool(int b);
void xin_println(void);

// Printf
void xin_printf(const char* format, ...);
```

## 5. 测试用例

### 5.1 字符串拼接

```xin
// 基本拼接
func test_basic_concat() {
    let s = "Hello" + " " + "World"
    println(s)  // 输出: Hello World
}

// 与数字拼接
func test_number_concat() {
    let a = "Value: " + 42
    let b = 100 + " points"
    println(a)  // 输出: Value: 42
    println(b)  // 输出: 100 points
}

// 与布尔值拼接
func test_bool_concat() {
    let s = "Flag is " + true
    println(s)  // 输出: Flag is true
}

// 边界情况：空字符串
func test_empty_string_concat() {
    let a = "" + ""
    let b = "" + "hello"
    let c = "world" + ""
    println(a)  // 输出: (空行)
    println(b)  // 输出: hello
    println(c)  // 输出: world
}

// 特殊浮点值
func test_special_float_concat() {
    let nan_str = "Value: " + (0.0 / 0.0)
    let inf_str = "Max: " + (1.0 / 0.0)
    println(nan_str)  // 输出: Value: NaN
    println(inf_str)  // 输出: Max: Infinity
}
```

### 5.2 打印函数

```xin
func test_print() {
    print("No newline")
    print(" ")
    print("here")
    println("")  // 输出: No newline here
}

func test_printf() {
    printf("Int: %d, Float: %.2f\n", 42, 3.14159)
    // 输出: Int: 42, Float: 3.14

    printf("String: %s, Bool: %b\n", "test", true)
    // 输出: String: test, Bool: true

    printf("Hex: 0x%x, Octal: %o\n", 255, 64)
    // 输出: Hex: 0xff, Octal: 100
}

// 边界情况：printf 特殊格式
func test_printf_edge_cases() {
    // 孤立的 %
    printf("100%%\n")  // 输出: 100%

    // 末尾孤立 %
    printf("test%\n")  // 输出: test%

    // 空字符串
    printf("")  // 无输出

    // 宽度与精度边界
    printf("%5d\n", 42)    // 输出: "   42"
    printf("%-5dend\n", 42) // 输出: "42   end"
    printf("%.0f\n", 3.9)  // 输出: "4" (四舍五入)
}
```

### 5.3 编译期错误测试

```xin
// 错误：printf 参数数量不匹配
func test_printf_arg_count_error() {
    printf("%d %s\n", 42)  // 编译错误: expected 2 arguments, found 1
}

// 错误：printf 类型不匹配
func test_printf_type_error() {
    printf("%d\n", "hello")  // 编译错误: expected 'int' for '%d', found 'string'
}

// 错误：未知占位符
func test_printf_unknown_placeholder() {
    printf("%z\n", 42)  // 编译错误: unknown format specifier '%z'
}
```

## 6. 未来扩展

### 6.1 字符串插值

考虑在未来支持字符串插值语法：

```xin
let name = "Alice"
let age = 30
println("$name is $age years old")
// 输出: Alice is 30 years old
```

### 6.2 更多格式化选项

- `%e` - 科学计数法
- `%p` - 指针地址
- 自定义格式化函数

## 7. 风险与限制

### 7.1 内存安全

- 当前设计中，字符串拼接产生的新字符串需要手动管理内存
- MVP 阶段采用简单策略，未来需要集成到编译期 GC 系统

### 7.2 性能

- 多次拼接会产生多个临时字符串
- 未来可考虑字符串构建器优化