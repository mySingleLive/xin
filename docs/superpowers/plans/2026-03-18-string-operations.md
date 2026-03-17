# 字符串操作与内置打印函数实现计划

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 Xin 语言的字符串拼接操作和内置打印函数（println、print、printf），包括编译期内存管理。

**Architecture:** 扩展 IR 添加 StringConcat 和 StringFree 指令；类型检查器添加 printf 格式检查和字符串 + 类型推断；IR Builder 处理字符串拼接和 printf 调用；代码生成器生成调用运行时函数的代码；运行时实现字符串拼接、释放和格式化打印函数。

**Tech Stack:** Rust, Cranelift, C runtime

---

## 文件结构

| 文件 | 职责 |
|-----|------|
| `crates/xin-ir/src/ir.rs` | IR 定义，添加 StringConcat, StringFree, ConcatType |
| `crates/xin-ir/src/builder.rs` | IR 生成，处理字符串拼接和 printf |
| `crates/xin-semantic/src/type_check.rs` | 类型检查，printf 格式验证，字符串 + 类型推断 |
| `crates/xin-codegen/src/aot.rs` | 代码生成，处理新 IR 指令 |
| `runtime/runtime.c` | 运行时函数实现 |
| `tests/integration_test.rs` | 集成测试 |

---

## Chunk 1: 运行时函数实现

### Task 1: 添加字符串拼接和释放运行时函数

**Files:**
- Modify: `runtime/runtime.c`

- [ ] **Step 1: 添加字符串拼接函数**

在 `runtime/runtime.c` 末尾添加字符串拼接和释放函数：

```c
#include <stdlib.h>
#include <string.h>
#include <math.h>

// String concatenation: string + string
char* xin_str_concat_ss(const char* a, const char* b) {
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
```

- [ ] **Step 2: 添加 printf 运行时函数**

在 `runtime/runtime.c` 末尾添加：

```c
#include <stdarg.h>

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
                    int len = 4; // "true" or "false" max length
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
                default: {
                    // Rewind to handle standard format specifiers
                    p--;
                    // Use vprintf for standard specifiers
                    // For simplicity, we'll handle common cases
                    if (*p == 'd' || *p == 'i' || *p == 'x' || *p == 'X' || *p == 'o' || *p == 'c') {
                        long long val = va_arg(args, long long);
                        printf("%lld", val);
                    } else if (*p == 'f') {
                        double val = va_arg(args, double);
                        printf("%g", val);
                    } else if (*p == 's') {
                        const char* val = va_arg(args, const char*);
                        printf("%s", val ? val : "(null)");
                    }
                }
            }
            p++;
        } else {
            putchar(*p);
            p++;
        }
    }

    va_end(args);
}
```

- [ ] **Step 3: 验证编译**

运行: `cargo build`
预期: 编译成功

- [ ] **Step 4: 提交**

```bash
git add runtime/runtime.c
git commit -m "feat(runtime): add string concatenation, free and printf functions"
```

---

## Chunk 2: IR 定义扩展

### Task 2: 添加 StringConcat 和 StringFree IR 指令

**Files:**
- Modify: `crates/xin-ir/src/ir.rs`

- [ ] **Step 1: 添加 ConcatType 枚举**

在 `IRType` 枚举定义之后添加：

```rust
/// Type of operand in string concatenation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConcatType {
    String,
    Int,
    Float,
    Bool,
}
```

- [ ] **Step 2: 添加新 IR 指令**

在 `Instruction` 枚举中，在 `Phi` 变体之后添加：

```rust
    /// String concatenation: %result = concat left, right
    StringConcat {
        result: Value,
        left: Value,
        left_type: ConcatType,
        right: Value,
        right_type: ConcatType,
    },

    /// String deallocation: free value
    StringFree {
        value: Value,
    },
```

- [ ] **Step 3: 验证编译**

运行: `cargo build`
预期: 编译成功（可能有未使用的警告）

- [ ] **Step 4: 提交**

```bash
git add crates/xin-ir/src/ir.rs
git commit -m "feat(ir): add StringConcat and StringFree instructions"
```

### Task 3: 导出 ConcatType

**Files:**
- Modify: `crates/xin-ir/src/lib.rs`

- [ ] **Step 1: 确保 ConcatType 被导出**

`lib.rs` 应该已经有 `pub use ir::*;`，所以 `ConcatType` 会自动导出。验证：

运行: `cargo build`
预期: 编译成功

---

## Chunk 3: 类型检查器修改

### Task 4: 完善字符串 + 类型检查

**Files:**
- Modify: `crates/xin-semantic/src/type_check.rs`

- [ ] **Step 1: 修改 Binary Add 类型检查**

定位到 `ExprKind::Binary` 处理代码（约 389-406 行），修改 `BinOp::Add` 的处理：

```rust
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        // String concatenation: if either side is string, result is string
                        if left_type == Type::String || right_type == Type::String {
                            if *op == BinOp::Add {
                                // Allow string concatenation with any basic type
                                match (&left_type, &right_type) {
                                    (Type::String, Type::String)
                                    | (Type::String, Type::Int)
                                    | (Type::String, Type::Float)
                                    | (Type::String, Type::Bool)
                                    | (Type::Int, Type::String)
                                    | (Type::Float, Type::String)
                                    | (Type::Bool, Type::String) => {
                                        return Ok(Type::String);
                                    }
                                    _ => {
                                        return Err(SemanticError::TypeMismatch {
                                            expected: Type::String,
                                            found: right_type,
                                        });
                                    }
                                }
                            }
                        }
                        // Numeric operations
                        if left_type == Type::Int && right_type == Type::Int {
                            Ok(Type::Int)
                        } else if left_type == Type::Float || right_type == Type::Float {
                            Ok(Type::Float)
                        } else {
                            Err(SemanticError::TypeMismatch {
                                expected: left_type.clone(),
                                found: right_type,
                            })
                        }
                    }
```

- [ ] **Step 2: 验证编译**

运行: `cargo build`
预期: 编译成功

- [ ] **Step 3: 提交**

```bash
git add crates/xin-semantic/src/type_check.rs
git commit -m "feat(semantic): improve string concatenation type checking"
```

### Task 5: 添加 printf 格式字符串验证

**Files:**
- Modify: `crates/xin-semantic/src/type_check.rs`

- [ ] **Step 1: 在文件顶部添加需要的 use**

确保有：
```rust
use xin_ast::*;
```

- [ ] **Step 2: 添加 printf 格式解析辅助函数**

在 `TypeChecker` impl 块中添加：

```rust
    /// Parse printf format string and return expected types
    fn parse_printf_format(&self, format: &str) -> Result<Vec<Type>, SemanticError> {
        let mut types = Vec::new();
        let chars: Vec<char> = format.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '%' {
                i += 1;
                if i >= chars.len() {
                    // Trailing % - not an error, just output %
                    break;
                }

                // Skip width/precision modifiers
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == '-') {
                    i += 1;
                }

                if i >= chars.len() {
                    break;
                }

                match chars[i] {
                    '%' => {} // Escaped %, no argument
                    'd' | 'i' | 'x' | 'X' | 'o' | 'c' | 'l' => {
                        types.push(Type::Int);
                    }
                    'f' => {
                        types.push(Type::Float);
                    }
                    's' => {
                        types.push(Type::String);
                    }
                    'b' => {
                        types.push(Type::Bool);
                    }
                    unknown => {
                        return Err(SemanticError::InvalidFormatSpecifier(unknown));
                    }
                }
            }
            i += 1;
        }

        Ok(types)
    }
```

- [ ] **Step 3: 添加新的 SemanticError 变体**

打开 `crates/xin-semantic/src/error.rs`，在 `SemanticError` 枚举中添加：

```rust
    InvalidFormatSpecifier(char),
    PrintfArgumentCountMismatch { expected: usize, found: usize },
    PrintfArgumentTypeMismatch { expected: Type, found: Type, specifier: char },
```

- [ ] **Step 4: 修改 Call 处理添加 printf 检查**

在 `check_expr` 的 `ExprKind::Call` 处理中，在 `if name == "println" || name == "print"` 检查之后添加：

```rust
                    if name == "printf" {
                        if args.is_empty() {
                            return Err(SemanticError::WrongNumberOfArguments {
                                expected: 1,
                                found: 0,
                            });
                        }
                        // Check first argument is a string literal
                        if let ExprKind::StringLiteral(format_str) = &args[0].kind {
                            let expected_types = self.parse_printf_format(format_str)?;
                            if args.len() - 1 != expected_types.len() {
                                return Err(SemanticError::PrintfArgumentCountMismatch {
                                    expected: expected_types.len(),
                                    found: args.len() - 1,
                                });
                            }
                            for (arg, expected_type) in args[1..].iter().zip(expected_types.iter()) {
                                let arg_type = self.check_expr(arg)?;
                                if !self.types_compatible(expected_type, &arg_type) {
                                    return Err(SemanticError::PrintfArgumentTypeMismatch {
                                        expected: expected_type.clone(),
                                        found: arg_type,
                                        specifier: 'X', // Generic
                                    });
                                }
                            }
                            return Ok(Type::Void);
                        } else {
                            // Non-literal format string, check it's a string type
                            let format_type = self.check_expr(&args[0])?;
                            if format_type != Type::String {
                                return Err(SemanticError::TypeMismatch {
                                    expected: Type::String,
                                    found: format_type,
                                });
                            }
                            // Can't validate format at compile time, just check remaining args
                            for arg in &args[1..] {
                                self.check_expr(arg)?;
                            }
                            return Ok(Type::Void);
                        }
                    }
```

- [ ] **Step 5: 更新 error.rs 的 Display 实现**

在 `error.rs` 的 `Display` impl 中添加新错误类型的处理：

```rust
            SemanticError::InvalidFormatSpecifier(c) => {
                write!(f, "unknown format specifier '%{}'", c)
            }
            SemanticError::PrintfArgumentCountMismatch { expected, found } => {
                write!(f, "printf argument count mismatch: expected {}, found {}", expected, found)
            }
            SemanticError::PrintfArgumentTypeMismatch { expected, found, specifier: _ } => {
                write!(f, "printf argument type mismatch: expected {:?}, found {:?}", expected, found)
            }
```

- [ ] **Step 6: 注册 printf 内置函数**

在 `register_builtins` 方法中添加：

```rust
        // printf: accepts format string and variable args, returns void
        let printf_symbol = Symbol {
            name: "printf".to_string(),
            kind: SymbolKind::Function {
                params: vec![("format".to_string(), Type::String, false)],
                return_type: Type::Void,
            },
            scope_level: 0,
        };
        self.scopes.define("printf", printf_symbol);
```

- [ ] **Step 7: 验证编译**

运行: `cargo build`
预期: 编译成功

- [ ] **Step 8: 提交**

```bash
git add crates/xin-semantic/src/error.rs crates/xin-semantic/src/type_check.rs
git commit -m "feat(semantic): add printf format string validation"
```

---

## Chunk 4: IR Builder 修改

### Task 6: 添加字符串拼接 IR 生成

**Files:**
- Modify: `crates/xin-ir/src/builder.rs`

- [ ] **Step 1: 导入 ConcatType**

在文件顶部确保导入：

```rust
use crate::{BinOp, ConcatType, ExternFunction, Instruction, IRFunction, IRModule, IRType, Value};
```

- [ ] **Step 2: 修改 Binary 表达式处理**

在 `build_expr` 方法的 `ExprKind::Binary` 分支中，修改为：

```rust
            ExprKind::Binary { op, left, right } => {
                let left_val = self.build_expr(left)?;
                let right_val = self.build_expr(right)?;

                // Check if this is string concatenation
                let left_type = Self::get_expr_type(left);
                let right_type = Self::get_expr_type(right);

                if *op == AstBinOp::Add {
                    let is_string_concat = matches!(left_type, Some(Type::String))
                        || matches!(right_type, Some(Type::String));

                    if is_string_concat {
                        let result = self.new_temp();
                        self.emit(Instruction::StringConcat {
                            result: result.clone(),
                            left: left_val,
                            left_type: self.type_to_concat_type(&left_type),
                            right: right_val,
                            right_type: self.type_to_concat_type(&right_type),
                        });
                        return Some(result);
                    }
                }

                // Regular binary operation
                let result = self.new_temp();
                self.emit(Instruction::Binary {
                    result: result.clone(),
                    op: self.convert_binop(op),
                    left: left_val,
                    right: right_val,
                });
                Some(result)
            }
```

- [ ] **Step 3: 添加辅助方法**

在 `IRBuilder` impl 块中添加：

```rust
    /// Convert Type to ConcatType
    fn type_to_concat_type(&self, ty: &Option<Type>) -> ConcatType {
        match ty {
            Some(Type::Int) => ConcatType::Int,
            Some(Type::Float) => ConcatType::Float,
            Some(Type::Bool) => ConcatType::Bool,
            Some(Type::String) | None => ConcatType::String,
            _ => ConcatType::String,
        }
    }
```

- [ ] **Step 4: 添加字符串拼接外部函数声明辅助方法**

在 `IRBuilder` impl 块中添加：

```rust
    /// Declare string concat extern function if needed
    fn declare_str_concat_extern(&mut self, left_type: ConcatType, right_type: ConcatType) {
        let func_name = match (left_type, right_type) {
            (ConcatType::String, ConcatType::String) => "xin_str_concat_ss",
            (ConcatType::String, ConcatType::Int) => "xin_str_concat_si",
            (ConcatType::Int, ConcatType::String) => "xin_str_concat_is",
            (ConcatType::String, ConcatType::Float) => "xin_str_concat_sf",
            (ConcatType::Float, ConcatType::String) => "xin_str_concat_fs",
            (ConcatType::String, ConcatType::Bool) => "xin_str_concat_sb",
            (ConcatType::Bool, ConcatType::String) => "xin_str_concat_bs",
            _ => "xin_str_concat_ss", // fallback
        };

        let param_types = match (left_type, right_type) {
            (ConcatType::String, ConcatType::String) => vec![IRType::Ptr("char".to_string()), IRType::Ptr("char".to_string())],
            (ConcatType::String, ConcatType::Int) => vec![IRType::Ptr("char".to_string()), IRType::I64],
            (ConcatType::Int, ConcatType::String) => vec![IRType::I64, IRType::Ptr("char".to_string())],
            (ConcatType::String, ConcatType::Float) => vec![IRType::Ptr("char".to_string()), IRType::F64],
            (ConcatType::Float, ConcatType::String) => vec![IRType::F64, IRType::Ptr("char".to_string())],
            (ConcatType::String, ConcatType::Bool) => vec![IRType::Ptr("char".to_string()), IRType::Bool],
            (ConcatType::Bool, ConcatType::String) => vec![IRType::Bool, IRType::Ptr("char".to_string())],
            _ => vec![IRType::Ptr("char".to_string()), IRType::Ptr("char".to_string())],
        };

        self.declare_extern_if_needed(func_name, param_types, Some(IRType::Ptr("char".to_string())));
    }
```

- [ ] **Step 5: 验证编译**

运行: `cargo build`
预期: 编译成功

- [ ] **Step 6: 提交**

```bash
git add crates/xin-ir/src/builder.rs
git commit -m "feat(ir): add string concatenation IR generation"
```

### Task 7: 添加 printf IR 生成

**Files:**
- Modify: `crates/xin-ir/src/builder.rs`

- [ ] **Step 1: 在 build_expr 的 Call 处理中添加 printf**

在 `if name == "println"` 和 `else if name == "print"` 检查之后添加：

```rust
                        } else if name == "printf" {
                            return self.handle_printf(args);
```

- [ ] **Step 2: 添加 handle_printf 方法**

在 `IRBuilder` impl 块中添加：

```rust
    /// Handle printf(format, args...) - formatted print
    fn handle_printf(&mut self, args: &[Expr]) -> Option<Value> {
        if args.is_empty() {
            return None;
        }

        // Build all arguments
        let arg_vals: Vec<Value> = args.iter().filter_map(|a| self.build_expr(a)).collect();

        // Call xin_printf
        self.emit(Instruction::Call {
            result: None,
            func: "xin_printf".to_string(),
            args: arg_vals,
            is_extern: true,
        });

        // Declare external function
        // xin_printf takes: const char* format, ... (variadic)
        self.declare_extern_if_needed("xin_printf", vec![IRType::Ptr("char".to_string())], None);

        None
    }
```

- [ ] **Step 3: 验证编译**

运行: `cargo build`
预期: 编译成功

- [ ] **Step 4: 提交**

```bash
git add crates/xin-ir/src/builder.rs
git commit -m "feat(ir): add printf IR generation"
```

---

## Chunk 5: 代码生成器修改

### Task 8: 添加 StringConcat 代码生成

**Files:**
- Modify: `crates/xin-codegen/src/aot.rs`

- [ ] **Step 1: 导入 ConcatType**

在文件顶部修改导入：

```rust
use xin_ir::{BinOp, ConcatType, ExternFunction, Instruction, IRFunction, IRModule, IRType};
```

- [ ] **Step 2: 在 compile_instruction 中添加 StringConcat 处理**

在 `Instruction::Binary` 处理之后添加：

```rust
            Instruction::StringConcat { result, left, right, left_type, right_type } => {
                // Determine which runtime function to call
                let func_name = match (left_type, right_type) {
                    (ConcatType::String, ConcatType::String) => "xin_str_concat_ss",
                    (ConcatType::String, ConcatType::Int) => "xin_str_concat_si",
                    (ConcatType::Int, ConcatType::String) => "xin_str_concat_is",
                    (ConcatType::String, ConcatType::Float) => "xin_str_concat_sf",
                    (ConcatType::Float, ConcatType::String) => "xin_str_concat_fs",
                    (ConcatType::String, ConcatType::Bool) => "xin_str_concat_sb",
                    (ConcatType::Bool, ConcatType::String) => "xin_str_concat_bs",
                    _ => "xin_str_concat_ss",
                };

                let left_val = self.load_variable(builder, left, variables)?;
                let right_val = self.load_variable(builder, right, variables)?;

                // Get or create the function reference
                let func_ref = if let Some(fr) = func_ref_cache.get(func_name) {
                    *fr
                } else {
                    let func_id = *self.extern_func_ids.get(func_name)
                        .expect("String concat function should be declared");
                    let sig = self.func_sigs.get(func_name)
                        .expect("Signature should exist")
                        .clone();
                    let sig_ref = builder.func.import_signature(sig);
                    let user_func_name = builder.func.declare_imported_user_function(
                        cranelift::codegen::ir::UserExternalName {
                            namespace: 0,
                            index: func_id.as_u32(),
                        }
                    );
                    let fr = builder.import_function(cranelift::codegen::ir::ExtFuncData {
                        name: cranelift::codegen::ir::ExternalName::user(user_func_name),
                        signature: sig_ref,
                        colocated: true,
                    });
                    func_ref_cache.insert(func_name.to_string(), fr);
                    fr
                };

                let call_val = builder.ins().call(func_ref, &[left_val, right_val]);
                let ret_val = builder.inst_results(call_val)[0];
                self.store_variable(builder, result, ret_val, variables, var_counter, self.pointer_type);
            }
```

- [ ] **Step 3: 添加 StringFree 处理**

在 StringConcat 之后添加：

```rust
            Instruction::StringFree { value } => {
                let val = self.load_variable(builder, value, variables)?;

                let func_ref = if let Some(fr) = func_ref_cache.get("xin_str_free") {
                    *fr
                } else {
                    let func_id = *self.extern_func_ids.get("xin_str_free")
                        .expect("xin_str_free should be declared");
                    let sig = self.func_sigs.get("xin_str_free")
                        .expect("Signature should exist")
                        .clone();
                    let sig_ref = builder.func.import_signature(sig);
                    let user_func_name = builder.func.declare_imported_user_function(
                        cranelift::codegen::ir::UserExternalName {
                            namespace: 0,
                            index: func_id.as_u32(),
                        }
                    );
                    let fr = builder.import_function(cranelift::codegen::ir::ExtFuncData {
                        name: cranelift::codegen::ir::ExternalName::user(user_func_name),
                        signature: sig_ref,
                        colocated: true,
                    });
                    func_ref_cache.insert("xin_str_free".to_string(), fr);
                    fr
                };

                builder.ins().call(func_ref, &[val]);
            }
```

- [ ] **Step 4: 验证编译**

运行: `cargo build`
预期: 编译成功

- [ ] **Step 5: 提交**

```bash
git add crates/xin-codegen/src/aot.rs
git commit -m "feat(codegen): add StringConcat and StringFree code generation"
```

---

## Chunk 6: 集成测试

### Task 9: 添加字符串拼接测试

**Files:**
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: 添加字符串拼接类型检查测试**

在文件末尾添加：

```rust
#[test]
fn test_string_concat_type_check() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_semantic::TypeChecker;

    // String + String
    let source = r#"
        func main() {
            let s = "Hello" + " World"
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();
    let mut type_checker = TypeChecker::new();
    assert!(type_checker.check(&ast).is_ok());

    // String + Int
    let source = r#"
        func main() {
            let s = "Value: " + 42
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();
    let mut type_checker = TypeChecker::new();
    assert!(type_checker.check(&ast).is_ok());

    // Int + String
    let source = r#"
        func main() {
            let s = 100 + " points"
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();
    let mut type_checker = TypeChecker::new();
    assert!(type_checker.check(&ast).is_ok());

    // String + Float
    let source = r#"
        func main() {
            let s = "Pi = " + 3.14
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();
    let mut type_checker = TypeChecker::new();
    assert!(type_checker.check(&ast).is_ok());

    // String + Bool
    let source = r#"
        func main() {
            let s = "Flag: " + true
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();
    let mut type_checker = TypeChecker::new();
    assert!(type_checker.check(&ast).is_ok());
}
```

- [ ] **Step 2: 添加 printf 类型检查测试**

```rust
#[test]
fn test_printf_type_check() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_semantic::TypeChecker;

    // Valid printf
    let source = r#"
        func main() {
            printf("Int: %d, Float: %f\n", 42, 3.14)
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();
    let mut type_checker = TypeChecker::new();
    assert!(type_checker.check(&ast).is_ok());

    // String format
    let source = r#"
        func main() {
            printf("Hello %s\n", "World")
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();
    let mut type_checker = TypeChecker::new();
    assert!(type_checker.check(&ast).is_ok());

    // Bool format (%b)
    let source = r#"
        func main() {
            printf("Flag: %b\n", true)
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();
    let mut type_checker = TypeChecker::new();
    assert!(type_checker.check(&ast).is_ok());
}

#[test]
fn test_printf_error_wrong_arg_count() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_semantic::TypeChecker;

    let source = r#"
        func main() {
            printf("%d %s\n", 42)
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();
    let mut type_checker = TypeChecker::new();
    assert!(type_checker.check(&ast).is_err());
}

#[test]
fn test_printf_error_type_mismatch() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_semantic::TypeChecker;

    let source = r#"
        func main() {
            printf("%d\n", "hello")
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();
    let mut type_checker = TypeChecker::new();
    assert!(type_checker.check(&ast).is_err());
}
```

- [ ] **Step 3: 添加 IR 生成测试**

```rust
#[test]
fn test_string_concat_ir() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_semantic::TypeChecker;
    use xin_ir::IRBuilder;

    let source = r#"
        func main() {
            let s = "Hello" + " World"
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();

    let mut type_checker = TypeChecker::new();
    type_checker.check(&ast).unwrap();

    let mut ir_builder = IRBuilder::new();
    let ir_module = ir_builder.build(&ast);
    assert_eq!(ir_module.functions.len(), 1);

    // Check that xin_str_concat_ss is declared
    assert!(ir_module.extern_functions.iter().any(|f| f.name == "xin_str_concat_ss"));
}
```

- [ ] **Step 4: 运行测试验证**

运行: `cargo test`
预期: 所有测试通过

- [ ] **Step 5: 提交**

```bash
git add tests/integration_test.rs
git commit -m "test: add string concat and printf integration tests"
```

---

## Chunk 7: 端到端测试

### Task 10: 创建可运行的测试程序

**Files:**
- Create: `examples/string_test.xin`

- [ ] **Step 1: 创建测试示例文件**

创建文件 `examples/string_test.xin`：

```xin
// string_test.xin
func main() {
    // Basic string concatenation
    let s1 = "Hello" + " " + "World"
    println(s1)

    // String + Int
    let s2 = "Count: " + 42
    println(s2)

    // Int + String
    let s3 = 100 + " points"
    println(s3)

    // String + Float
    let s4 = "Pi = " + 3.14159
    println(s4)

    // String + Bool
    let s5 = "Flag: " + true
    println(s5)

    // printf tests
    printf("Integer: %d\n", 42)
    printf("Float: %f\n", 3.14)
    printf("String: %s\n", "test")
    printf("Bool: %b\n", true)
    printf("Hex: 0x%x\n", 255)
}
```

- [ ] **Step 2: 编译并运行测试**

运行: `cargo run -- compile examples/string_test.xin -o string_test && ./string_test`
预期输出:
```
Hello World
Count: 42
100 points
Pi = 3.14159
Flag: true
Integer: 42
Float: 3.14
String: test
Bool: true
Hex: 0xff
```

注意：实际运行可能需要调整命令或修复遗漏的实现。

- [ ] **Step 3: 提交**

```bash
git add examples/string_test.xin
git commit -m "test: add string operations example"
```

---

## 总结

完成以上所有任务后，Xin 语言将支持：

1. **字符串拼接**：`"Hello" + " World"`，`"Count: " + 42`，`3.14 + " is pi"` 等
2. **内置打印函数**：
   - `println(value)` - 打印任意类型并换行
   - `print(value)` - 打印任意类型不换行
   - `printf(format, args...)` - 格式化打印，支持 %d, %f, %s, %b, %x 等占位符
3. **编译期类型检查**：printf 格式字符串与参数类型匹配验证

**后续工作（未包含在本计划中）：**
- 字符串内存管理（编译期 GC）
- 作用域结束时的字符串释放
- 控制流路径的释放代码插入