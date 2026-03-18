# Xin 编译器端到端测试套件设计

**日期**: 2026-03-18

## 1. 概述

### 1.1 目标

为 Xin 编译器创建端到端测试套件，通过编写多个 xin 语言代码文件覆盖所有已实现的语法特性，编译并运行这些代码，验证输出是否符合预期。

### 1.2 前置条件

在运行完整测试套件之前，编译器需要满足以下条件：

| 前置条件 | 状态 | 说明 |
|---------|------|------|
| 控制流代码生成 | ⚠️ 待实现 | `Jump`, `Branch`, `Label` 指令需在 AOT 代码生成中实现 |
| 函数返回值传递 | ⚠️ 待修复 | 函数返回值需正确传递给调用者 |

**当前限制**：
- if/else 分支目前会顺序执行（所有分支都执行）
- for 循环目前无法正常工作
- 递归函数返回值可能不正确

**分阶段测试策略**：
- 第一阶段：测试不依赖控制流的特性（basic/, strings/, operators/）
- 第二阶段：待控制流实现后，测试 control_flow/, functions/ 目录

### 1.3 范围

仅测试当前编译器已实现的语法特性：

| 特性类别 | 支持情况 | 代码生成状态 |
|---------|---------|-------------|
| 基本类型 (int, float, bool, string) | ✅ 已实现 | ✅ 已实现 |
| 变量声明 (let) | ✅ 已实现 | ✅ 已实现 |
| 函数定义与调用 | ✅ 已实现 | ⚠️ 返回值待修复 |
| if/else 语句 | ✅ 已实现 | ⚠️ 待实现 |
| for 循环 (C风格, while, 无限循环) | ✅ 已实现 | ⚠️ 待实现 |
| 二元运算 (+, -, *, /, %, ==, !=, <, >, <=, >=, &&, \|\|) | ✅ 已实现 | ✅ 已实现 |
| 一元运算 (-, !) | ✅ 已实现 | ✅ 已实现 |
| 打印函数 (println, print, printf) | ✅ 已实现 | ✅ 已实现 |
| 字符串拼接 | ✅ 已实现 | ✅ 已实现 |
| 条件表达式 (三元运算符) | ✅ 已实现 | ⚠️ 依赖控制流 |

**不在范围内**：struct, interface, List/Map, 空安全操作, 指针类型, 所有权系统, Lambda 表达式, 模块系统等尚未实现的特性。

## 2. 测试框架设计

### 2.1 目录结构

```
tests/
├── run_tests.sh              # 测试运行脚本
├── basic/                    # 基础功能测试
│   ├── arithmetic.xin
│   ├── arithmetic.expected
│   ├── variables.xin
│   ├── variables.expected
│   ├── types.xin
│   └── types.expected
├── control_flow/             # 控制流测试（第二阶段启用）
│   ├── if_else.xin
│   ├── if_else.expected
│   ├── for_loops.xin
│   └── for_loops.expected
├── functions/                # 函数测试（第二阶段启用）
│   ├── basic_funcs.xin
│   ├── basic_funcs.expected
│   ├── recursion.xin
│   └── recursion.expected
├── strings/                  # 字符串测试
│   ├── concat.xin
│   ├── concat.expected
│   ├── printf.xin
│   └── printf.expected
└── operators/                # 运算符测试
    ├── comparison.xin
    ├── comparison.expected
    ├── logical.xin
    ├── logical.expected
    ├── unary.xin
    └── unary.expected
```

### 2.2 测试运行脚本

**文件**: `tests/run_tests.sh`

**功能要求**:

1. 遍历所有测试目录，找到 `.xin` 测试文件
2. 对每个测试文件：
   - 编译 `.xin` 文件为可执行文件
   - 运行可执行文件，捕获输出和退出码
   - 读取对应的 `.expected` 文件
   - 比对实际输出与预期输出
3. 遇到失败立即停止，显示详细错误信息
4. 输出测试进度和结果摘要
5. 忽略编译器警告信息（如链接器警告），只关注程序实际输出

**命令行参数**:

| 参数 | 说明 |
|-----|------|
| 无参数 | 运行所有测试（默认跳过第二阶段测试目录） |
| `<目录名>` | 只运行指定目录的测试 |
| `--all` | 运行所有测试（包括第二阶段目录） |
| `-v, --verbose` | 显示详细输出 |
| `-h, --help` | 显示帮助信息 |

**注**: 第一阶段测试目录为 `basic/`、`strings/`、`operators/`。第二阶段目录 `control_flow/`、`functions/` 默认跳过，待编译器修复后使用 `--all` 参数运行。

**输出格式**:

成功时:
```
[Xin E2E Tests]

Running basic/arithmetic... ✓
Running basic/variables... ✓
...

All tests passed! (8/8)
```

失败时:
```
[Xin E2E Tests]

Running basic/arithmetic... ✓
Running basic/variables... ✗ FAILED

--- Expected ---
42

--- Actual ---
0

Test failed: basic/variables
Stopped at first failure.

Summary: 1 passed, 1 failed
```

### 2.3 测试验证规则

**验证步骤**:

1. **编译验证**: 编译命令执行成功（退出码为 0）
2. **运行验证**: 可执行文件运行成功（退出码为 0）
3. **输出验证**: 实际输出与 `.expected` 文件内容完全匹配

**输出匹配规则**:

- 按行比对，去除每行末尾空白字符
- 精确匹配（不使用正则表达式）
- 区分大小写

## 3. 测试用例设计

### 3.1 basic/ 目录

#### 3.1.1 arithmetic.xin

**测试目标**: 验证整数和浮点数的算术运算

**测试内容**:
- 整数加、减、乘、除、取模
- 浮点数加、减、乘、除
- 运算符优先级
- 混合运算

**预期输出**（基于编译器实际行为校准）:
```
15
7
20
5
2
15.5
7.5
20
5
14
```

#### 3.1.2 variables.xin

**测试目标**: 验证变量声明和类型推断

**测试内容**:
- let 声明整数、浮点数、布尔值、字符串
- 变量使用和输出
- 变量重新赋值

**预期输出**:
```
42
3.14
true
hello
100
```

#### 3.1.3 types.xin

**测试目标**: 验证基本类型的打印输出

**测试内容**:
- 打印各类型的字面量
- 打印各类型的变量

**预期输出**:
```
42
3.14
true
false
hello world
100
2.5
false
variable
```

### 3.2 control_flow/ 目录（第二阶段）

#### 3.2.1 if_else.xin

**测试目标**: 验证 if/else 条件语句

**测试内容**:
- 基本 if 语句
- if-else 语句
- if-else if-else 语句
- 嵌套 if 语句
- 条件表达式（三元运算符）

**预期输出**（待控制流实现后验证）:
```
greater
equal
nested
ten
```

#### 3.2.2 for_loops.xin

**测试目标**: 验证各种形式的 for 循环

**测试内容**:
- C 风格 for 循环
- while 风格 for 循环
- 无限循环（带 break）

**预期输出**（待控制流实现后验证）:
```
0
1
2
3
4
count: 5
5
4
3
2
1
done
```

### 3.3 functions/ 目录（第二阶段）

#### 3.3.1 basic_funcs.xin

**测试目标**: 验证函数定义、调用、参数传递、返回值

**测试内容**:
- 无参数无返回值函数
- 有参数无返回值函数
- 有参数有返回值函数
- 多参数函数
- 函数内调用函数

**预期输出**（待函数返回值修复后验证）:
```
hello
greeting: Alice
result: 15
sum: 30
nested: 42
```

#### 3.3.2 recursion.xin

**测试目标**: 验证递归函数

**测试内容**:
- 斐波那契数列
- 阶乘计算

**预期输出**（待控制流和函数返回值修复后验证）:
```
55
120
```

### 3.4 strings/ 目录

#### 3.4.1 concat.xin

**测试目标**: 验证字符串拼接操作

**测试内容**:
- 字符串 + 字符串
- 字符串 + 整数
- 整数 + 字符串
- 字符串 + 浮点数
- 字符串 + 布尔值

**预期输出**（基于 string_test.xin 实际运行结果）:
```
Hello World
Count: 42
100 points
Pi = 3.14159
Flag: true
```

#### 3.4.2 printf.xin

**测试目标**: 验证 printf 格式化输出

**测试内容**:
- %d 整数格式化
- %f 浮点数格式化
- %s 字符串格式化
- %b 布尔值格式化
- %x 十六进制格式化
- %o 八进制格式化

**预期输出**（基于 string_test.xin 实际运行结果）:
```
Integer: 42
Float: 3.14
String: test
Bool: true
Hex: 0xff
Octal: 100
```

### 3.5 operators/ 目录

#### 3.5.1 comparison.xin

**测试目标**: 验证比较运算符

**测试内容**:
- ==, != 相等比较
- <, >, <=, >= 大小比较
- 整数比较

**预期输出**:
```
true
false
true
false
true
false
true
false
true
true
false
true
true
false
```

#### 3.5.2 logical.xin

**测试目标**: 验证逻辑运算符

**测试内容**:
- && 逻辑与
- || 逻辑或
- ! 逻辑非

**预期输出**:
```
true
false
false
true
true
true
```

#### 3.5.3 unary.xin

**测试目标**: 验证一元运算符

**测试内容**:
- - 负号
- ! 逻辑非

**预期输出**:
```
-42
42
false
true
```

## 4. 实现步骤

### 4.1 第一阶段：基础测试

| 步骤 | 内容 |
|-----|------|
| 1 | 创建 tests/ 目录结构 |
| 2 | 编写 run_tests.sh 测试脚本 |
| 3 | 编写 basic/ 目录测试用例 |
| 4 | 编写 strings/ 目录测试用例 |
| 5 | 编写 operators/ 目录测试用例 |
| 6 | 运行第一阶段测试，验证通过 |

### 4.2 第二阶段：控制流和函数测试（待编译器修复后）

| 步骤 | 内容 |
|-----|------|
| 7 | 实现控制流代码生成（Jump, Branch, Label） |
| 8 | 修复函数返回值传递 |
| 9 | 编写 control_flow/ 目录测试用例 |
| 10 | 编写 functions/ 目录测试用例 |
| 11 | 运行完整测试套件，验证所有测试通过 |

## 5. 后续扩展

当编译器实现更多特性时，测试套件可按以下方式扩展：

| 新特性 | 新增测试文件 |
|-------|-------------|
| struct | `tests/structs/basic_structs.xin` |
| interface | `tests/interfaces/basic_impl.xin` |
| List/Map | `tests/collections/list.xin`, `tests/collections/map.xin` |
| 空安全 | `tests/null_safety/option.xin` |
| Lambda | `tests/functions/lambda.xin` |
| 模块系统 | `tests/modules/import.xin` |
| break/continue | `tests/control_flow/loop_control.xin` |