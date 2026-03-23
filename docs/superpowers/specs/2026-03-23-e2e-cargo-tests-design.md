---
name: e2e-cargo-tests
description: 将 xin 语法特性测试转换为 cargo 单元测试
type: project
---

# 设计方案：Xin 语法特性 E2E 测试

## 概述

将 `/Users/dt_flys/Projects/xin/tests` 目录下的 `.xin` 语法特性测试转换为 cargo 单元测试，实现端到端验证。

## 需求

- **验证层面**：端到端（编译 → 执行 → 对比输出）
- **组织方式**：每个 `.xin` 文件一个独立的 `#[test]` 函数
- **失败行为**：显示期望/实际输出差异，测试立即失败
- **文件位置**：追加到 `tests/integration_test.rs`

## 测试命名规范

```rust
#[test]
fn e2e_<目录>_<文件名>() { ... }
```

示例：
- `tests/basic/arithmetic.xin` → `e2e_basic_arithmetic()`
- `tests/functions/basic_funcs.xin` → `e2e_functions_basic_funcs()`

## 测试实现逻辑

每个测试函数执行以下步骤：

1. 构造 `.xin` 源文件路径
2. 调用编译器编译到临时二进制文件
3. 执行二进制文件获取实际输出
4. 读取对应的 `.expected` 文件获取期望输出
5. 比较输出（去除行尾空白后使用 `assert_eq!` 比较）

## 测试覆盖范围

预计生成 26 个测试函数：

| 目录 | 测试数量 | 测试文件 |
|------|----------|----------|
| `basic/` | 4 | arithmetic, type_conversion, types, variables |
| `strings/` | 2 | concat, printf |
| `operators/` | 3 | comparison, logical, unary |
| `templates/` | 3 | basic, expressions, escape |
| `control_flow/` | 4 | break_continue, for_loops, if_else, if_else_branching |
| `functions/` | 3 | basic_funcs, recursion, return_test |
| `arrays/` | 3 | basic, mutable, nested |
| `nullable/` | 1 | basic |
| `maps/` | 2 | basic, methods |
| `floats/` | 1 | arithmetic |

## 技术实现

### 编译器调用

使用 `std::process::Command` 调用 `cargo run -- compile <source> -o <output>` 进行编译。

### 错误处理

- **编译失败**：测试 panic 并显示编译器错误信息
- **执行失败**：测试 panic 并显示运行时错误信息
- **输出不匹配**：使用 `assert_eq!` 显示期望与实际输出的差异

### 临时文件处理

使用 `std::env::temp_dir()` 获取系统临时目录，生成唯一的临时二进制文件名。

### 输出比较

```rust
// 去除每行行尾空白后比较
fn normalize_output(s: &str) -> String {
    s.lines().map(|line| line.trim_end()).collect::<Vec<_>>().join("\n")
}
```

## 依赖

无需添加新依赖，使用 Rust 标准库即可。

## 文件变更

- 修改：`tests/integration_test.rs` - 追加 26 个测试函数