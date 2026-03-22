# Xin 语言语法特性实现计划

## 概述

本文档定义了 Xin 语言下一阶段的语法特性实现计划，涵盖类型系统重构、控制流修复、以及多项新增特性。

## 实现顺序

1. 类型系统重构
2. 控制流修复
3. break/continue 实现
4. 函数返回值修复
5. 类型转换
6. 空安全操作符
7. Lambda 表达式
8. Map 字面量

---

## 1. 类型系统重构

### 1.1 完整类型体系

#### 有符号整数

| 类型 | 位数 | 范围 |
|------|------|------|
| int8 | 8 | -128 ~ 127 |
| int16 | 16 | -32768 ~ 32767 |
| int32 | 32 | -2147483648 ~ 2147483647 |
| int64 | 64 | -9223372036854775808 ~ 9223372036854775807 |
| int128 | 128 | 更大范围 |

#### 无符号整数

| 类型 | 位数 | 范围 |
|------|------|------|
| uint8 | 8 | 0 ~ 255 |
| uint16 | 16 | 0 ~ 65535 |
| uint32 | 32 | 0 ~ 4294967295 |
| uint64 | 64 | 0 ~ 18446744073709551615 |
| uint128 | 128 | 更大范围 |
| byte | 8 | uint8 的别名 |

#### 浮点数

| 类型 | 位数 | 说明 |
|------|------|------|
| float8 | 8 | FP8 |
| float16 | 16 | 半精度 |
| float32 | 32 | 单精度 |
| float64 | 64 | 双精度 |
| float128 | 128 | 四精度 |

#### 其他类型

| 类型 | 说明 |
|------|------|
| char | Unicode 字符 |
| string | Unicode 字符串 |
| bool | 布尔值（true/false） |
| void | 无返回值 |
| object | 任意类型 |

### 1.2 字面量推断规则

| 字面量 | 推断类型 |
|--------|----------|
| 42 | int32 |
| 3.14 | float32 |
| true / false | bool |
| "hello" / 'hello' | string |
| """multi\nline""" | string（多行字符串） |

**无符号整数字面量**：必须通过类型注解指定
```xin
let x: uint32 = 42
let y: byte = 255
```

### 1.3 类型转换规则

**不支持隐式类型转换**。所有类型转换必须显式进行。

```xin
let a: int32 = 10
let b: int64 = 10      // 编译错误：类型不匹配
let b: int64 = int64(a) // 正确：显式转换
```

### 1.4 字符与字符串

#### 字符表示

使用 `char()` 函数从字符串创建字符：

```xin
char('A')      // 字符 'A'
char("中")     // Unicode 字符 '中'
char('😊')     // Emoji 字符
```

**char() 行为规则**：
- 字面量字符串：编译期检查长度必须为 1，否则报错
- 动态字符串：行为未定义（开发者需自行确保正确性）

#### 字符串表示

```xin
"hello"              // 双引号字符串
'hello'              // 单引号字符串（等价于双引号）
`hello {name}`       // 模板字符串
```

#### 多行字符串

使用三引号包裹，保留原样缩进：

```xin
"""line1
line2
line3"""

'''line1
line2
line3'''

```line1 {var}
line2```             // 多行模板字符串
```

### 1.5 影响范围

- `crates/xin-ast/src/ty.rs` — 类型定义
- `crates/xin-lexer/src/` — 新增关键字
- `crates/xin-parser/src/` — 类型解析
- `crates/xin-semantic/src/` — 类型检查和推断
- `crates/xin-ir/src/` — IR 类型表示
- `crates/xin-codegen/src/` — 代码生成
- `runtime/runtime.c` — 运行时函数
- `tests/` — 更新类型注解（int → int32, float → float32）

---

## 2. 控制流修复

### 2.1 当前问题

`crates/xin-codegen/src/cranelift.rs` 中控制流代码生成未实现跳转指令：
- if/else 分支顺序执行（没有条件跳转）
- for 循环无法正常工作（没有循环跳转）

### 2.2 解决方案

在 Cranelift codegen 中实现完整的控制流：

#### 基本块管理

每个控制流结构生成独立的基本块：

```
if/else: condition_block, then_block, else_block, merge_block
for: init_block, condition_block, body_block, update_block, exit_block
```

#### 跳转指令

- `br`: 无条件跳转
- `br_if`: 条件跳转
- `br_table`: switch 表达式（后续扩展）

#### if/else 示例

```
if (condition) { then_body } else { else_body }

→ condition_block:
    compute condition
    br_if condition, then_block, else_block

→ then_block:
    execute then_body
    br merge_block

→ else_block:
    execute else_body
    br merge_block

→ merge_block:
    continue...
```

#### for 循环基本块结构

**C 风格 for 循环：**

```
for (init; condition; update) { body }

→ init_block:
    execute init
    br condition_block

→ condition_block:
    compute condition
    br_if condition, body_block, exit_block

→ body_block:
    execute body
    br update_block

→ update_block:
    execute update
    br condition_block

→ exit_block:
    continue...
```

**while 风格 for 循环：**

```
for (condition) { body }

→ condition_block:
    compute condition
    br_if condition, body_block, exit_block

→ body_block:
    execute body
    br condition_block

→ exit_block:
    continue...
```

**无限循环：**

```
for { body }

→ body_block:
    execute body
    br body_block

// 通过 break 跳出
```

### 2.3 影响范围

- `crates/xin-codegen/src/cranelift.rs` — 主要修改
- `crates/xin-ir/src/builder.rs` — 可能需要 IR 层面的跳转指令支持

### 2.4 测试验证

现有 `tests/control_flow/` 目录的测试应能通过。

---

## 3. break/continue 实现

### 3.1 当前状态

AST 已定义 `Break` 和 `Continue` 语句，但 Codegen 未实现。

### 3.2 实现方案

#### IR 层

新增 IR 指令：
- `Break`: 跳出当前循环
- `Continue`: 跳到循环下一次迭代

#### Codegen 层

编译循环时维护循环的出口块和继续块：

```
for (init; condition; update) { body }

loop_exit_block: break 跳转目标
loop_continue_block: continue 跳转目标（跳到 update_block）

for-in 循环：
loop_exit_block: break 跳转目标
loop_continue_block: continue 跳转目标（跳到下一次迭代判断）
```

#### Semantic 层

跟踪"当前是否在循环中"，break/continue 只能在循环体内使用，否则报错。

### 3.3 影响范围

- `crates/xin-ast/src/stmt.rs` — 已有定义
- `crates/xin-semantic/src/` — 新增循环上下文检查
- `crates/xin-ir/src/` — 新增 Break/Continue IR 指令
- `crates/xin-codegen/src/cranelift.rs` — 实现跳转逻辑

---

## 4. 函数返回值修复

### 4.1 当前问题

递归函数返回值传递存在问题，可能是：
- 返回值寄存器使用不当
- 函数调用后栈状态不正确
- IR 层返回指令处理有误

### 4.2 解决方案

#### 诊断步骤

1. 编写最小复现用例（如递归阶乘）
2. 打印 IR 查看返回指令是否正确
3. 检查 Cranelift 函数调用约定

#### 可能的修复点

- 确保返回值通过正确的寄存器/栈位置传递
- 函数结束时正确设置返回值
- 递归调用时保存/恢复必要的寄存器

#### 验证测试

```xin
func factorial(n: int32) int32 {
    if (n <= 1) { return 1 }
    return n * factorial(n - 1)
}
println(factorial(5))  // 期望输出 120
```

### 4.3 影响范围

- `crates/xin-codegen/src/cranelift.rs` — 函数调用和返回指令
- `crates/xin-ir/src/` — 返回指令的 IR 表示（如需）

---

## 5. 类型转换

### 5.1 支持的转换

#### 基础类型转换函数

```xin
int8(x), int16(x), int32(x), int64(x), int128(x)
uint8(x), uint16(x), uint32(x), uint64(x), uint128(x)
float16(x), float32(x), float64(x), float128(x)
char(x)
string(x)
bool(x)
```

### 5.2 转换规则

| 源类型 → 目标类型 | 行为 |
|------------------|------|
| 整数 → 整数 | 截断或扩展（有符号符号扩展，无符号零扩展） |
| 整数 → 浮点 | 精确转换 |
| 浮点 → 整数 | 截断小数部分 |
| 浮点 → 浮点 | 精度转换 |
| char → string | 单字符字符串 |
| string → char | 编译期检查长度为 1 |
| 数值 → string | 格式化为字符串 |
| string → 数值 | 解析字符串，失败则运行时错误 |
| bool ↔ 数值 | true=1, false=0 |

### 5.3 示例

```xin
let a: int32 = 100
let b: int64 = int64(a)      // 整数扩展
let c: float32 = float32(a)  // 整数转浮点

let s: string = string(42)   // "42"
let n: int32 = int32("100")  // 100，解析失败则运行时错误

let ch: char = char('A')     // 字符 'A'
let str: string = string(ch) // "A"
```

### 5.4 影响范围

- `crates/xin-semantic/src/type_check.rs` — 类型转换合法性检查
- `crates/xin-codegen/src/cranelift.rs` — 转换指令生成
- `runtime/runtime.c` — 可能需要新增字符串转换函数

---

## 6. 空安全操作符

### 6.1 操作符定义

| 操作符 | 名称 | 说明 |
|--------|------|------|
| `?.` | 安全导航 | 如果对象为 null，返回 null 而非报错 |
| `??` | Elvis | 如果左侧为 null，返回右侧值 |

### 6.2 可空类型

```xin
string?    // 可空字符串，可能是 string 或 null
int32?     // 可空整数
char?      // 可空字符
```

**`?` 结合优先级：**

`?` 紧跟在类型后面，优先级最高：

```xin
int32?      // (int32)?，可空整数
int32?[]    // ((int32)?)[]，可空整数的数组
int32[]?    // (int32[])?，可空的整数数组
```

### 6.3 null 关键字

```xin
let name: string? = null
```

### 6.4 安全导航 ?.

```xin
let user: User? = get_user()
let city = user?.address?.city  // 链式调用，结果为 string?
```

链式 `?.` 调用后，结果类型自动包装为可空类型。

### 6.5 Elvis 操作符 ??

```xin
let name: string? = null
let display = name ?? "未知"  // display: string

let count: int32? = null
let total = count ?? 0        // total: int32
```

`??` 操作符右侧的类型必须与左侧解包后的类型兼容，结果为非空类型。

### 6.6 可空类型语义

#### 赋值规则

```xin
let a: int32 = null        // 编译错误：非空类型不能赋值为 null
let b: int32? = null       // 正确
let c: int32? = 42         // 正确：自动包装为可空类型
```

#### 运算规则

可空类型不能直接参与运算，必须先解包：

```xin
let x: int32? = 10
let y = x + 1              // 编译错误：可空类型需先解包
let z = (x ?? 0) + 1       // 正确：使用 ?? 解包后运算
```

#### 比较操作

```xin
let a: int32? = 10
let b: int32? = null
(a ?? 0) == (b ?? 0)        // 正确：解包后比较
```

### 6.7 不实现的特性

- `!!` 强制解包操作符（可使用 `?? throw Error("xxx")` 替代，throw 表达式延后实现）

### 6.8 影响范围

- `crates/xin-ast/src/ty.rs` — 可空类型定义
- `crates/xin-ast/src/expr.rs` — 安全导航、Elvis 表达式
- `crates/xin-lexer/src/` — null 关键字
- `crates/xin-parser/src/` — 语法解析
- `crates/xin-semantic/src/` — 可空类型检查
- `crates/xin-ir/src/` — IR 指令
- `crates/xin-codegen/src/` — null 检查和跳转

---

## 7. Lambda 表达式

### 7.1 类型表示

```xin
func(int32, int32) int32    // 接受两个 int32，返回 int32
func(string) void           // 接受 string，无返回值
func() bool                 // 无参数，返回 bool
```

### 7.2 语法

#### 完整语法

```xin
(a: int32, b: int32) -> { return a + b }
```

#### 类型推断

```xin
(a, b) -> { return a + b }
```

#### 简写（单表达式）

```xin
(a, b) -> a + b
```

#### 单参数简写

```xin
a -> a * 2
```

### 7.3 Lambda 作为参数

#### 函数参数定义

```xin
func apply(arr: int32[], f: func(int32) int32) {
    for (var i = 0; i < arr.len(); i++) {
        arr[i] = f(arr[i])
    }
}
```

#### 普通传入

```xin
apply([1, 3], a -> a * 2 + 1)
```

#### 尾参 Lambda（多行）

```xin
apply([1, 4], a: int32) {
    return a * 2 + 1
}
```

#### 作为唯一参数传入

```xin
run(a: int32, b: int32) {
    return a + b * 2
}
// 等价于
run((a: int32, b: int32) -> { return a + b * 2 })
```

#### 无参作为尾参传入

```xin
forEach([1, 4]) {
    println("Processing")
}
```

#### 尾参 Lambda 解析规则

当函数调用的最后一个参数是 Lambda 时，可以将 Lambda 写在括号外：

```xin
// 以下两种写法等价：
apply([1, 4], a: int32) { return a * 2 + 1 }
apply([1, 4], (a: int32) -> { return a * 2 + 1 })

// 唯一参数的 Lambda 可以省略括号：
run(a: int32, b: int32) { return a + b * 2 }
run((a: int32, b: int32) -> { return a + b * 2 })
```

**解析规则：**
1. 函数调用后紧跟 `{` 且前一个参数是 Lambda 类型 → 尾参 Lambda
2. 函数调用后紧跟 `(params) {` → 唯一参数 Lambda
3. 括号闭合后紧跟 `{` → 尾参 Lambda

### 7.4 值捕获

Lambda 捕获变量的当前值，捕获后原变量修改不影响 Lambda 内的值：

```xin
let multiplier = 10
let multiply = x -> x * multiplier

multiplier = 20
multiply(5)  // 返回 50，不是 100
```

**捕获语义：**
- 在 Lambda 定义时捕获变量的值（快照）
- 捕获的是值本身，不是引用
- 嵌套 Lambda 各自独立捕获外层变量

### 7.5 影响范围

- `crates/xin-ast/src/ty.rs` — Lambda 类型定义
- `crates/xin-ast/src/expr.rs` — Lambda 表达式
- `crates/xin-parser/src/` — Lambda 语法解析
- `crates/xin-semantic/src/` — 类型推断、捕获分析
- `crates/xin-ir/src/` — Lambda IR 表示
- `crates/xin-codegen/src/` — Lambda 代码生成（闭包对象）

---

## 8. Map 字面量

### 8.1 语法

#### 字面量创建

```xin
let m = {"name": "Alice", "age": 30}   // 字符串字面量作为键
let m2 = {key: value}                   // 变量 key 的值作为键
let m3 = {getKey(): getValue()}         // 表达式作为键
let empty = {}                          // 空 Map
```

#### 键的规则

**用引号区分字面量和变量：**

| 语法 | 含义 |
|------|------|
| `{"name": value}` | 键是字符串字面量 `"name"` |
| `{key: value}` | 键是变量 `key` 的值 |
| `{expr(): value}` | 键是表达式 `expr()` 的返回值 |

**键类型限制：**

Map 的键类型目前仅支持 `string` 类型。整数或其他类型作为键的特性待后续扩展。

**示例：**

```xin
let id = "user123"
let user = {id: "Alice", "age": 30}
// user 的内容：
// - 键 "user123"（变量 id 的值）对应值 "Alice"
// - 键 "age"（字符串字面量）对应值 30

let key = "name"
let m = {key: "Bob"}
// m 的内容：键 "name"（变量 key 的值）对应值 "Bob"
```

#### 空 Map 与空代码块歧义

空 `{}` 根据上下文推断：
- **表达式位置**：解析为空 Map
- **语句位置**：解析为空代码块

```xin
let m = {}           // 表达式位置 → 空 Map
if (true) {}         // 语句位置 → 空代码块
func foo() {}        // 语句位置 → 空函数体
let x = {}           // 表达式位置 → 空 Map
```

### 8.2 类型推断

```xin
{"a": 1, "b": 2}              // map<string, int32>
{"a": 1, "b": "hello"}        // map<string, object>
{}                             // map<string, object>
```

### 8.3 访问方式

#### 索引访问

```xin
m["name"]            // 字符串字面量键
m[key]               // 变量键
m[getKey()]          // 表达式键
```

#### 点号访问

适用于所有实现了 HasKey 接口的对象（Map 接口继承自 HasKey）：

```xin
m."name"             // 双引号字符串键
m.'name'             // 单引号字符串键
m.`prefix_{key}`     // 模板字符串键
```

### 8.4 API

```xin
m.len()             // 键值对数量
m.keys()            // 所有键（返回数组）
m.values()          // 所有值（返回数组）
m.has("name")       // 检查键是否存在
m.remove("age")     // 删除键值对
```

### 8.5 HasKey 接口（未来实现）

```xin
interface HasKey {
    func get(key: string) object?
    func set(key: string, value: object)
    func has(key: string) bool
}
```

**注意**：接口特性属于低优先级，在接口实现前，Map 点号访问先只在 Map 类型上支持。

### 8.6 实现策略

采用分阶段实现：
1. **第一阶段**：动态类型 Map，值类型为 object
2. **第二阶段**：泛型系统就绪后，迁移到静态类型 `Map<K, V>`

### 8.7 影响范围

- `crates/xin-ast/src/ty.rs` — Map 类型定义
- `crates/xin-ast/src/expr.rs` — Map 字面量表达式
- `crates/xin-parser/src/` — Map 语法解析
- `crates/xin-semantic/src/` — Map 类型检查
- `crates/xin-ir/src/` — Map IR 表示
- `crates/xin-codegen/src/` — Map 代码生成
- `runtime/runtime.c` — Map 运行时支持

---

## 附录：待实现的低优先级特性

以下特性不在本次实现范围内：

- Struct 结构体
- Interface 接口（包括 HasKey）
- 模块系统
- 泛型类型
- 所有权系统 + move 语义
- 生命周期/借用检查
- 指针类型
- throw/try-catch 异常处理
- 标准库