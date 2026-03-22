# Xin 语法特性实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 Xin 语言的类型系统重构、控制流修复及多项新特性

**Architecture:** 编译器采用分层架构：Lexer → Parser → AST → Semantic → IR → Codegen。每个特性遵循 TDD 流程：先写失败测试 → 实现代码 → 测试通过。

**Tech Stack:** Rust, Cranelift (代码生成), 自定义运行时 (C)

**Spec:** `docs/superpowers/specs/2026-03-23-syntax-features-implementation-plan.md`

---

## 文件结构

### 类型系统重构

| 文件 | 修改类型 | 说明 |
|------|---------|------|
| `crates/xin-ast/src/ty.rs` | 修改 | 新增数值类型定义 |
| `crates/xin-ast/src/token.rs` | 修改 | 新增类型关键字 Token |
| `crates/xin-lexer/src/lexer.rs` | 修改 | 新增关键字识别、多行字符串 |
| `crates/xin-parser/src/parser.rs` | 修改 | 类型解析逻辑 |
| `crates/xin-semantic/src/type_check.rs` | 修改 | 类型检查和推断 |
| `crates/xin-ir/src/ir.rs` | 修改 | IR 类型定义 |
| `crates/xin-codegen/src/cranelift.rs` | 修改 | 类型代码生成 |
| `tests/basic/types.xin` | 修改 | 更新测试用例 |

### 控制流修复

| 文件 | 修改类型 | 说明 |
|------|---------|------|
| `crates/xin-ir/src/builder.rs` | 修改 | 块管理和跳转指令 |
| `crates/xin-codegen/src/cranelift.rs` | 修改 | 实现跳转指令 |
| `tests/control_flow/*.xin` | 修改 | 验证测试通过 |

---

## 里程碑 1: 类型系统重构 - AST 层

### Task 1.1: 定义新的数值类型

**Files:**
- Modify: `crates/xin-ast/src/ty.rs`

- [ ] **Step 1: 编写失败测试**

在 `ty.rs` 底部添加测试模块：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_integer_types() {
        assert_eq!(Type::Int8.to_string(), "int8");
        assert_eq!(Type::Int16.to_string(), "int16");
        assert_eq!(Type::Int32.to_string(), "int32");
        assert_eq!(Type::Int64.to_string(), "int64");
        assert_eq!(Type::Int128.to_string(), "int128");
        assert_eq!(Type::UInt8.to_string(), "uint8");
        assert_eq!(Type::UInt16.to_string(), "uint16");
        assert_eq!(Type::UInt32.to_string(), "uint32");
        assert_eq!(Type::UInt64.to_string(), "uint64");
        assert_eq!(Type::UInt128.to_string(), "uint128");
    }

    #[test]
    fn test_new_float_types() {
        assert_eq!(Type::Float8.to_string(), "float8");
        assert_eq!(Type::Float16.to_string(), "float16");
        assert_eq!(Type::Float32.to_string(), "float32");
        assert_eq!(Type::Float64.to_string(), "float64");
        assert_eq!(Type::Float128.to_string(), "float128");
    }

    #[test]
    fn test_char_type() {
        assert_eq!(Type::Char.to_string(), "char");
    }

    #[test]
    fn test_type_helpers() {
        assert!(Type::Int32.is_signed_integer());
        assert!(Type::UInt32.is_unsigned_integer());
        assert!(Type::Int32.is_integer());
        assert!(Type::Float32.is_float());
        assert!(Type::Int32.is_numeric());
        assert_eq!(Type::Int32.integer_bit_width(), Some(32));
        assert_eq!(Type::UInt64.integer_bit_width(), Some(64));
    }
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p xin-ast -- test_new_integer_types test_new_float_types test_char_type test_type_helpers`
Expected: FAIL - Type 枚举没有 Int8, Float8, Char 等变体

- [ ] **Step 3: 修改 Type 枚举**

将 `Int` 和 `Float` 替换为具体位宽的类型：

```rust
/// Type representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    // Signed integers
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    // Unsigned integers
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    // Floating point
    Float8,
    Float16,
    Float32,
    Float64,
    Float128,
    // Other types
    Bool,
    Char,
    String,
    Void,
    Object,
    // User-defined type name
    Named(String),
    // Pointer type: *T or *mut T
    Pointer {
        inner: Box<Type>,
        mutable: bool,
    },
    // Nullable type: T?
    Nullable(Box<Type>),
    // Array type: T[]
    Array(Box<Type>),
    // Generic type: List<T>, Map<K, V>
    Generic {
        name: String,
        args: Vec<Type>,
    },
    // Function type: func(A, B) R
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
}
```

- [ ] **Step 4: 更新 Display trait 实现**

```rust
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int8 => write!(f, "int8"),
            Type::Int16 => write!(f, "int16"),
            Type::Int32 => write!(f, "int32"),
            Type::Int64 => write!(f, "int64"),
            Type::Int128 => write!(f, "int128"),
            Type::UInt8 => write!(f, "uint8"),
            Type::UInt16 => write!(f, "uint16"),
            Type::UInt32 => write!(f, "uint32"),
            Type::UInt64 => write!(f, "uint64"),
            Type::UInt128 => write!(f, "uint128"),
            Type::Float8 => write!(f, "float8"),
            Type::Float16 => write!(f, "float16"),
            Type::Float32 => write!(f, "float32"),
            Type::Float64 => write!(f, "float64"),
            Type::Float128 => write!(f, "float128"),
            Type::Char => write!(f, "char"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Void => write!(f, "void"),
            Type::Object => write!(f, "object"),
            // ... 其他保持不变
        }
    }
}
```

- [ ] **Step 5: 添加辅助方法**

```rust
impl Type {
    /// Check if this is a signed integer type
    pub fn is_signed_integer(&self) -> bool {
        matches!(self, Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64 | Type::Int128)
    }

    /// Check if this is an unsigned integer type
    pub fn is_unsigned_integer(&self) -> bool {
        matches!(self, Type::UInt8 | Type::UInt16 | Type::UInt32 | Type::UInt64 | Type::UInt128)
    }

    /// Check if this is any integer type
    pub fn is_integer(&self) -> bool {
        self.is_signed_integer() || self.is_unsigned_integer()
    }

    /// Check if this is a floating point type
    pub fn is_float(&self) -> bool {
        matches!(self, Type::Float8 | Type::Float16 | Type::Float32 | Type::Float64 | Type::Float128)
    }

    /// Check if this is a numeric type
    pub fn is_numeric(&self) -> bool {
        self.is_integer() || self.is_float()
    }

    /// Get the bit width of integer types
    pub fn integer_bit_width(&self) -> Option<u32> {
        match self {
            Type::Int8 | Type::UInt8 => Some(8),
            Type::Int16 | Type::UInt16 => Some(16),
            Type::Int32 | Type::UInt32 => Some(32),
            Type::Int64 | Type::UInt64 => Some(64),
            Type::Int128 | Type::UInt128 => Some(128),
            _ => None,
        }
    }
}
```

- [ ] **Step 6: 运行测试确认通过**

Run: `cargo test -p xin-ast`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add crates/xin-ast/src/ty.rs
git commit -m "feat(ast): add new numeric types (int8-128, uint8-128, float8-128, char)"
```

---

### Task 1.2: 添加类型关键字 Token

**Files:**
- Modify: `crates/xin-ast/src/token.rs`

- [ ] **Step 1: 编写失败测试**

在 `token.rs` 底部添加测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_type_tokens() {
        assert_eq!(TokenKind::Int8.to_string(), "int8");
        assert_eq!(TokenKind::Int32.to_string(), "int32");
        assert_eq!(TokenKind::UInt64.to_string(), "uint64");
        assert_eq!(TokenKind::Byte.to_string(), "byte");
        assert_eq!(TokenKind::Float8.to_string(), "float8");
        assert_eq!(TokenKind::Float32.to_string(), "float32");
        assert_eq!(TokenKind::Char.to_string(), "char");
    }
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p xin-ast -- test_new_type_tokens`
Expected: FAIL - TokenKind 没有这些变体

- [ ] **Step 3: 添加新的 Token 变体**

修改 `TokenKind` 枚举，替换 `Int` 和 `Float`：

```rust
pub enum TokenKind {
    // ... existing variants (Literals, Identifiers...)

    // Types - Signed integers
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    // Types - Unsigned integers
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    Byte,
    // Types - Floating point
    Float8,
    Float16,
    Float32,
    Float64,
    Float128,
    // Types - Other
    Char,
    Bool,
    String,
    Void,

    // ... existing operators and special tokens
}
```

- [ ] **Step 4: 更新 Display trait**

```rust
impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Int8 => write!(f, "int8"),
            TokenKind::Int16 => write!(f, "int16"),
            TokenKind::Int32 => write!(f, "int32"),
            TokenKind::Int64 => write!(f, "int64"),
            TokenKind::Int128 => write!(f, "int128"),
            TokenKind::UInt8 => write!(f, "uint8"),
            TokenKind::UInt16 => write!(f, "uint16"),
            TokenKind::UInt32 => write!(f, "uint32"),
            TokenKind::UInt64 => write!(f, "uint64"),
            TokenKind::UInt128 => write!(f, "uint128"),
            TokenKind::Byte => write!(f, "byte"),
            TokenKind::Float8 => write!(f, "float8"),
            TokenKind::Float16 => write!(f, "float16"),
            TokenKind::Float32 => write!(f, "float32"),
            TokenKind::Float64 => write!(f, "float64"),
            TokenKind::Float128 => write!(f, "float128"),
            TokenKind::Char => write!(f, "char"),
            // ... 其他保持不变
        }
    }
}
```

- [ ] **Step 5: 运行测试确认通过**

Run: `cargo test -p xin-ast -- test_new_type_tokens`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/xin-ast/src/token.rs
git commit -m "feat(ast): add token kinds for new numeric types"
```

---

## 里程碑 1: 类型系统重构 - Lexer 层

### Task 1.3: Lexer 识别新关键字

**Files:**
- Modify: `crates/xin-lexer/src/lexer.rs`

- [ ] **Step 1: 编写失败测试**

```rust
#[test]
fn test_new_type_keywords() {
    let input = "int8 int16 int32 int64 int128 uint8 uint16 uint32 uint64 uint128 byte float8 float16 float32 float64 float128 char";
    let tokens = tokenize(input);

    let expected = vec![
        TokenKind::Int8, TokenKind::Int16, TokenKind::Int32, TokenKind::Int64, TokenKind::Int128,
        TokenKind::UInt8, TokenKind::UInt16, TokenKind::UInt32, TokenKind::UInt64, TokenKind::UInt128,
        TokenKind::Byte,
        TokenKind::Float8, TokenKind::Float16, TokenKind::Float32, TokenKind::Float64, TokenKind::Float128,
        TokenKind::Char,
    ];

    for (i, token) in tokens.iter().enumerate() {
        assert_eq!(token.kind, expected[i]);
    }
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p xin-lexer -- test_new_type_keywords`
Expected: FAIL - 关键字未被识别

- [ ] **Step 3: 添加关键字映射**

在关键字识别部分添加新类型关键字：

```rust
// 在 match keyword { ... } 中添加：
"int8" => TokenKind::Int8,
"int16" => TokenKind::Int16,
"int32" => TokenKind::Int32,
"int64" => TokenKind::Int64,
"int128" => TokenKind::Int128,
"uint8" => TokenKind::UInt8,
"uint16" => TokenKind::UInt16,
"uint32" => TokenKind::UInt32,
"uint64" => TokenKind::UInt64,
"uint128" => TokenKind::UInt128,
"byte" => TokenKind::Byte,
"float8" => TokenKind::Float8,
"float16" => TokenKind::Float16,
"float32" => TokenKind::Float32,
"float64" => TokenKind::Float64,
"float128" => TokenKind::Float128,
"char" => TokenKind::Char,
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p xin-lexer -- test_new_type_keywords`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/xin-lexer/src/lexer.rs
git commit -m "feat(lexer): recognize new type keywords"
```

---

### Task 1.4: 实现多行字符串

**Files:**
- Modify: `crates/xin-lexer/src/lexer.rs`

- [ ] **Step 1: 编写失败测试**

```rust
#[test]
fn test_multiline_string() {
    let input = r#"""line1
line2
line3""""#;
    let tokens = tokenize(input);
    assert_eq!(tokens.len(), 2); // StringLiteral + EOF
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
    assert_eq!(tokens[0].text, "line1\nline2\nline3");
}

#[test]
fn test_multiline_template_string() {
    let input = r#"```hello
{name}```"#;
    let tokens = tokenize(input);
    assert_eq!(tokens[0].kind, TokenKind::TemplateString);
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p xin-lexer -- test_multiline`
Expected: FAIL - 多行字符串未被正确识别

- [ ] **Step 3: 实现多行字符串词法分析**

在字符串解析逻辑中添加三引号支持：

```rust
fn scan_string(&mut self, quote: char) -> Result<Token, LexerError> {
    let start = self.position;

    // 检查是否是三引号
    let is_triple = self.peek_next() == Some(quote) && self.peek_next_next() == Some(quote);

    if is_triple {
        // 跳过三个引号
        self.advance();
        self.advance();
        self.advance();

        let mut content = String::new();
        // 读取直到遇到结束的三引号
        while !self.is_at_end() {
            if self.peek() == Some(quote)
                && self.peek_next() == Some(quote)
                && self.peek_next_next() == Some(quote) {
                // 跳过结束的三引号
                self.advance();
                self.advance();
                self.advance();
                break;
            }
            content.push(self.advance().unwrap());
        }

        return Ok(Token::new(
            TokenKind::StringLiteral,
            content,
            self.line,
            self.column,
        ));
    }

    // 原有的单引号字符串逻辑...
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p xin-lexer -- test_multiline`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/xin-lexer/src/lexer.rs
git commit -m "feat(lexer): implement multiline strings with triple quotes"
```

---

## 里程碑 1: 类型系统重构 - Parser 层

### Task 1.5: Parser 解析新类型

**Files:**
- Modify: `crates/xin-parser/src/parser.rs`

- [ ] **Step 1: 编写失败测试**

```rust
#[test]
fn test_parse_new_types() {
    let cases = vec![
        ("let x: int32 = 0", Type::Int32),
        ("let x: uint64 = 0", Type::UInt64),
        ("let x: float8 = 0.0", Type::Float8),
        ("let x: float32 = 0.0", Type::Float32),
        ("let x: byte = 0", Type::UInt8), // byte is alias
        ("let x: char = char('a')", Type::Char),
    ];

    for (input, expected_type) in cases {
        let result = parse(input);
        assert!(result.is_ok());
        // 验证变量类型为 expected_type
    }
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p xin-parser -- test_parse_new_types`
Expected: FAIL - 新类型未被解析

- [ ] **Step 3: 更新类型解析函数**

```rust
fn parse_type(&mut self) -> Result<Type, ParseError> {
    let token = self.peek()?;

    let base_type = match token.kind {
        TokenKind::Int8 => Type::Int8,
        TokenKind::Int16 => Type::Int16,
        TokenKind::Int32 => Type::Int32,
        TokenKind::Int64 => Type::Int64,
        TokenKind::Int128 => Type::Int128,
        TokenKind::UInt8 => Type::UInt8,
        TokenKind::UInt16 => Type::UInt16,
        TokenKind::UInt32 => Type::UInt32,
        TokenKind::UInt64 => Type::UInt64,
        TokenKind::UInt128 => Type::UInt128,
        TokenKind::Byte => Type::UInt8, // byte is alias for uint8
        TokenKind::Float8 => Type::Float8,
        TokenKind::Float16 => Type::Float16,
        TokenKind::Float32 => Type::Float32,
        TokenKind::Float64 => Type::Float64,
        TokenKind::Float128 => Type::Float128,
        TokenKind::Char => Type::Char,
        TokenKind::Bool => Type::Bool,
        TokenKind::String => Type::String,
        TokenKind::Void => Type::Void,
        // ... existing cases
    };

    self.advance()?;

    // Handle nullable (?), array ([]), etc.
    // ... existing logic
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p xin-parser -- test_parse_new_types`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/xin-parser/src/parser.rs
git commit -m "feat(parser): parse new type annotations"
```

---

## 里程碑 1: 类型系统重构 - Semantic 层

### Task 1.6: 类型推断规则

**Files:**
- Modify: `crates/xin-semantic/src/type_check.rs`

- [ ] **Step 1: 编写失败测试**

```rust
#[test]
fn test_literal_inference_int() {
    // 整数字面量推断为 int32
    let result = check_program("let x = 42");
    assert!(result.is_ok());
    // 验证 x 的类型为 int32
}

#[test]
fn test_literal_inference_float() {
    // 浮点字面量推断为 float32
    let result = check_program("let x = 3.14");
    assert!(result.is_ok());
    // 验证 x 的类型为 float32
}

#[test]
fn test_no_implicit_conversion() {
    // int32 不能隐式转换为 int64
    let result = check_program("let x: int64 = 42"); // 42 是 int32
    assert!(result.is_err()); // 应该报类型不匹配错误
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p xin-semantic -- test_literal_inference test_no_implicit_conversion`
Expected: FAIL - 推断规则未实现

- [ ] **Step 3: 更新字面量类型推断**

```rust
fn infer_literal_type(&self, literal: &Literal) -> Type {
    match literal {
        Literal::Int(_) => Type::Int32,      // 整数字面量默认 int32
        Literal::Float(_) => Type::Float32,  // 浮点字面量默认 float32
        Literal::Bool(_) => Type::Bool,
        Literal::String(_) => Type::String,
    }
}
```

- [ ] **Step 4: 更新类型兼容性检查**

```rust
fn check_type_compatibility(&self, expected: &Type, actual: &Type) -> bool {
    // 不支持隐式转换，类型必须完全匹配
    expected == actual
}
```

- [ ] **Step 5: 运行测试确认通过**

Run: `cargo test -p xin-semantic -- test_literal_inference test_no_implicit_conversion`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/xin-semantic/src/type_check.rs
git commit -m "feat(semantic): implement type inference rules (int32/float32 default)"
```

---

## 里程碑 1: 类型系统重构 - IR 层

### Task 1.7: IR 类型定义

**Files:**
- Modify: `crates/xin-ir/src/ir.rs`

- [ ] **Step 1: 编写失败测试**

```rust
#[test]
fn test_ir_type_conversion() {
    assert_eq!(IRType::from_ast_type(&Type::Int8), IRType::I8);
    assert_eq!(IRType::from_ast_type(&Type::Int32), IRType::I32);
    assert_eq!(IRType::from_ast_type(&Type::UInt64), IRType::U64);
    assert_eq!(IRType::from_ast_type(&Type::Float8), IRType::F8);
    assert_eq!(IRType::from_ast_type(&Type::Float32), IRType::F32);
    assert_eq!(IRType::from_ast_type(&Type::Char), IRType::Char);
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p xin-ir -- test_ir_type_conversion`
Expected: FAIL - IRType 变体不存在

- [ ] **Step 3: 更新 IRType 枚举**

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IRType {
    // Signed integers
    I8,
    I16,
    I32,
    I64,
    I128,
    // Unsigned integers
    U8,
    U16,
    U32,
    U64,
    U128,
    // Floating point
    F8,
    F16,
    F32,
    F64,
    F128,
    // Other
    Bool,
    Char,
    String,
    Void,
    Ptr(String),
    Object,
}
```

- [ ] **Step 4: 更新 Display trait**

```rust
impl fmt::Display for IRType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IRType::I8 => write!(f, "i8"),
            IRType::I16 => write!(f, "i16"),
            IRType::I32 => write!(f, "i32"),
            IRType::I64 => write!(f, "i64"),
            IRType::I128 => write!(f, "i128"),
            IRType::U8 => write!(f, "u8"),
            IRType::U16 => write!(f, "u16"),
            IRType::U32 => write!(f, "u32"),
            IRType::U64 => write!(f, "u64"),
            IRType::U128 => write!(f, "u128"),
            IRType::F8 => write!(f, "f8"),
            IRType::F16 => write!(f, "f16"),
            IRType::F32 => write!(f, "f32"),
            IRType::F64 => write!(f, "f64"),
            IRType::F128 => write!(f, "f128"),
            IRType::Char => write!(f, "char"),
            // ...
        }
    }
}
```

- [ ] **Step 5: 添加 AST Type 到 IRType 的转换**

```rust
impl IRType {
    pub fn from_ast_type(ty: &Type) -> Self {
        match ty {
            Type::Int8 => IRType::I8,
            Type::Int16 => IRType::I16,
            Type::Int32 => IRType::I32,
            Type::Int64 => IRType::I64,
            Type::Int128 => IRType::I128,
            Type::UInt8 => IRType::U8,
            Type::UInt16 => IRType::U16,
            Type::UInt32 => IRType::U32,
            Type::UInt64 => IRType::U64,
            Type::UInt128 => IRType::U128,
            Type::Float8 => IRType::F8,
            Type::Float16 => IRType::F16,
            Type::Float32 => IRType::F32,
            Type::Float64 => IRType::F64,
            Type::Float128 => IRType::F128,
            Type::Char => IRType::Char,
            Type::Bool => IRType::Bool,
            Type::String => IRType::String,
            Type::Void => IRType::Void,
            Type::Object => IRType::Object,
            // ...
        }
    }
}
```

- [ ] **Step 6: 运行测试确认通过**

Run: `cargo test -p xin-ir -- test_ir_type_conversion`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add crates/xin-ir/src/ir.rs
git commit -m "feat(ir): add IR types for new numeric types"
```

---

## 里程碑 1: 类型系统重构 - Codegen 层

### Task 1.8: 代码生成类型映射

**Files:**
- Modify: `crates/xin-codegen/src/cranelift.rs`

- [ ] **Step 1: 编写失败测试**

```rust
#[test]
fn test_type_mapping() {
    let gen = CodeGenerator::new().unwrap();
    assert_eq!(gen.convert_type(&IRType::I8), types::I8);
    assert_eq!(gen.convert_type(&IRType::I32), types::I32);
    assert_eq!(gen.convert_type(&IRType::U64), types::I64);
    assert_eq!(gen.convert_type(&IRType::F32), types::F32);
    assert_eq!(gen.convert_type(&IRType::Char), types::I32);
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p xin-codegen -- test_type_mapping`
Expected: FAIL - convert_type 方法不支持新类型

- [ ] **Step 3: 更新类型转换函数**

```rust
fn convert_type(&self, ty: &IRType) -> Type {
    match ty {
        IRType::I8 => types::I8,
        IRType::I16 => types::I16,
        IRType::I32 => types::I32,
        IRType::I64 => types::I64,
        IRType::I128 => types::I128,
        IRType::U8 => types::I8,
        IRType::U16 => types::I16,
        IRType::U32 => types::I32,
        IRType::U64 => types::I64,
        IRType::U128 => types::I128,
        IRType::F8 => types::F32,  // F8 使用 F32 存储
        IRType::F16 => types::F32, // F16 使用 F32 存储
        IRType::F32 => types::F32,
        IRType::F64 => types::F64,
        IRType::F128 => types::F128,
        IRType::Bool => types::I8,
        IRType::Char => types::I32, // Unicode code point
        IRType::String => self.module.target_config().pointer_type(),
        IRType::Void => types::VOID,
        IRType::Ptr(_) => self.module.target_config().pointer_type(),
        IRType::Object => self.module.target_config().pointer_type(),
    }
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p xin-codegen -- test_type_mapping`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/xin-codegen/src/cranelift.rs
git commit -m "feat(codegen): add code generation for new numeric types"
```

---

## 里程碑 1: 类型系统重构 - 测试更新

### Task 1.9: 更新现有测试

**Files:**
- Modify: `tests/basic/types.xin`
- Modify: `tests/basic/arithmetic.xin`
- Create: `tests/basic/multiline_strings.xin`

- [ ] **Step 1: 更新 types.xin 测试**

```xin
// tests/basic/types.xin

// 新的数值类型
let a: int32 = 42
let b: int64 = 100
let c: uint32 = 200
let d: byte = 255
let e: float32 = 3.14
let f: float64 = 3.14159265358979

// 字面量推断
let g = 42        // 推断为 int32
let h = 3.14      // 推断为 float32

println(a)
println(b)
println(c)
println(d)
println(e)
println(f)
```

- [ ] **Step 2: 创建多行字符串测试**

```xin
// tests/basic/multiline_strings.xin

let s1 = """line1
line2
line3"""

let s2 = '''hello
world'''

let name = "World"
let s3 = ```Hello
{name}!```

println(s1)
println(s2)
println(s3)
```

- [ ] **Step 3: 运行测试验证**

Run: `cargo test --test e2e -- types multiline`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add tests/basic/types.xin tests/basic/arithmetic.xin tests/basic/multiline_strings.xin
git commit -m "test: update tests for new type system and multiline strings"
```

---

### Task 1.10: 实现 char() 函数

**Files:**
- Modify: `crates/xin-semantic/src/type_check.rs`
- Modify: `crates/xin-codegen/src/cranelift.rs`
- Modify: `runtime/runtime.c`

- [ ] **Step 1: 编写失败测试**

```rust
#[test]
fn test_char_function_compile_time_check() {
    // 编译期检查长度为 1
    let result = check_program("let c = char('a')");
    assert!(result.is_ok());

    // 编译期检查长度不为 1 应报错
    let result = check_program("let c = char('ab')");
    assert!(result.is_err());
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p xin-semantic -- test_char_function`
Expected: FAIL - char() 函数检查未实现

- [ ] **Step 3: 实现 Semantic 层检查**

```rust
fn check_char_call(&mut self, arg: &Expr) -> Result<Type, SemanticError> {
    // 如果参数是字符串字面量，检查长度
    if let ExprKind::StringLiteral(s) = &arg.kind {
        if s.chars().count() != 1 {
            return Err(SemanticError::InvalidCharLiteral(s.clone()));
        }
    }
    Ok(Type::Char)
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p xin-semantic -- test_char_function`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/xin-semantic/src/type_check.rs crates/xin-codegen/src/cranelift.rs runtime/runtime.c
git commit -m "feat: implement char() function with compile-time length check"
```

---

## 里程碑 2: 控制流修复

### Task 2.1: 实现 IR 跳转指令

**Files:**
- Modify: `crates/xin-ir/src/builder.rs`

- [ ] **Step 1: 编写失败测试**

```rust
#[test]
fn test_ir_jump_and_branch() {
    let mut builder = IRBuilder::new();

    builder.create_block("entry");
    builder.create_block("then");
    builder.create_block("else");
    builder.create_block("merge");

    builder.switch_to_block("entry");
    let cond = builder.new_temp();
    builder.emit_const(cond.clone(), "1", IRType::Bool);
    builder.emit_branch(cond, "then", "else");

    builder.switch_to_block("then");
    builder.emit_jump("merge");

    builder.switch_to_block("else");
    builder.emit_jump("merge");

    // 验证指令序列
    assert!(builder.instructions().iter().any(|i| matches!(i, Instruction::Jump(_))));
    assert!(builder.instructions().iter().any(|i| matches!(i, Instruction::Branch { .. })));
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p xin-ir -- test_ir_jump_and_branch`
Expected: FAIL - 方法不存在

- [ ] **Step 3: 实现块管理和跳转指令**

```rust
pub struct IRBuilder {
    blocks: Vec<String>,
    current_block: Option<String>,
    instructions: Vec<Instruction>,
    temp_counter: usize,
}

impl IRBuilder {
    pub fn create_block(&mut self, label: &str) {
        self.blocks.push(label.to_string());
    }

    pub fn switch_to_block(&mut self, label: &str) {
        self.current_block = Some(label.to_string());
        self.instructions.push(Instruction::Label(label.to_string()));
    }

    pub fn emit_jump(&mut self, target: &str) {
        self.instructions.push(Instruction::Jump(target.to_string()));
    }

    pub fn emit_branch(&mut self, cond: Value, then_label: &str, else_label: &str) {
        self.instructions.push(Instruction::Branch {
            cond,
            then_label: then_label.to_string(),
            else_label: else_label.to_string(),
        });
    }
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p xin-ir -- test_ir_jump_and_branch`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/xin-ir/src/builder.rs
git commit -m "feat(ir): add block management and jump instructions"
```

---

### Task 2.2: 实现 if/else 代码生成

**Files:**
- Modify: `crates/xin-codegen/src/cranelift.rs`

- [ ] **Step 1: 编写失败测试**

创建 `tests/control_flow/if_else_branching.xin`:

```xin
// 测试 if/else 分支是否正确执行
let x = 10
let result = 0

if (x > 5) {
    result = 1
} else {
    result = 2
}

println(result)  // 期望输出: 1

if (x < 5) {
    result = 3
} else {
    result = 4
}

println(result)  // 期望输出: 4
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo run -- tests/control_flow/if_else_branching.xin`
Expected: FAIL - 输出不正确（分支顺序执行）

- [ ] **Step 3: 实现 if/else 代码生成**

```rust
fn compile_if_else(
    &mut self,
    builder: &mut FunctionBuilder,
    condition: &Expr,
    then_block: &Block,
    else_block: Option<&Block>,
    blocks: &mut BlockContext,
) -> Result<(), String> {
    let then_label = format!("then_{}", blocks.counter);
    let else_label = format!("else_{}", blocks.counter);
    let merge_label = format!("merge_{}", blocks.counter);
    blocks.counter += 1;

    // 创建基本块
    let then_block_cr = builder.create_block();
    let else_block_cr = builder.create_block();
    let merge_block_cr = builder.create_block();

    blocks.cranelift_blocks.insert(then_label.clone(), then_block_cr);
    blocks.cranelift_blocks.insert(else_label.clone(), else_block_cr);
    blocks.cranelift_blocks.insert(merge_label.clone(), merge_block_cr);

    // 编译条件
    let cond_val = self.compile_expr(builder, condition, blocks)?;
    builder.ins().brif(cond_val, then_block_cr, &[], else_block_cr, &[]);

    // Then 块
    builder.switch_to_block(then_block_cr);
    self.compile_block(builder, then_block, blocks)?;
    builder.ins().jump(merge_block_cr, &[]);

    // Else 块
    builder.switch_to_block(else_block_cr);
    if let Some(else_body) = else_block {
        self.compile_block(builder, else_body, blocks)?;
    }
    builder.ins().jump(merge_block_cr, &[]);

    // Merge 块
    builder.switch_to_block(merge_block_cr);

    // Seal blocks
    builder.seal_block(then_block_cr);
    builder.seal_block(else_block_cr);
    builder.seal_block(merge_block_cr);

    Ok(())
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo run -- tests/control_flow/if_else_branching.xin`
Expected: 输出 1 和 4

- [ ] **Step 5: Commit**

```bash
git add crates/xin-codegen/src/cranelift.rs tests/control_flow/if_else_branching.xin
git commit -m "feat(codegen): implement if/else branching"
```

---

### Task 2.3: 实现 for 循环代码生成

**Files:**
- Modify: `crates/xin-codegen/src/cranelift.rs`

- [ ] **Step 1: 编写失败测试**

创建 `tests/control_flow/for_loop_control.xin`:

```xin
// 测试 for 循环控制
let sum = 0
for (var i = 1; i <= 5; i = i + 1) {
    sum = sum + i
}
println(sum)  // 期望输出: 15

// while 风格
let count = 0
var n = 5
for (n > 0) {
    count = count + 1
    n = n - 1
}
println(count)  // 期望输出: 5
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo run -- tests/control_flow/for_loop_control.xin`
Expected: FAIL - 循环不正确

- [ ] **Step 3: 实现 for 循环代码生成**

```rust
fn compile_for_loop(
    &mut self,
    builder: &mut FunctionBuilder,
    init: &Option<Box<Stmt>>,
    condition: &Option<Expr>,
    update: &Option<Box<Stmt>>,
    body: &Block,
    blocks: &mut BlockContext,
) -> Result<(), String> {
    let loop_id = blocks.counter;
    blocks.counter += 1;

    let cond_label = format!("for_cond_{}", loop_id);
    let body_label = format!("for_body_{}", loop_id);
    let update_label = format!("for_update_{}", loop_id);
    let exit_label = format!("for_exit_{}", loop_id);

    // 创建块
    let cond_block = builder.create_block();
    let body_block = builder.create_block();
    let update_block = builder.create_block();
    let exit_block = builder.create_block();

    blocks.cranelift_blocks.insert(cond_label.clone(), cond_block);
    blocks.cranelift_blocks.insert(body_label.clone(), body_block);
    blocks.cranelift_blocks.insert(update_label.clone(), update_block);
    blocks.cranelift_blocks.insert(exit_label.clone(), exit_block);

    // 保存 break/continue 目标
    let saved_break = blocks.break_target.clone();
    let saved_continue = blocks.continue_target.clone();
    blocks.break_target = Some(exit_label.clone());
    blocks.continue_target = Some(update_label.clone());

    // Init
    if let Some(init_stmt) = init {
        self.compile_stmt(builder, init_stmt, blocks)?;
    }
    builder.ins().jump(cond_block, &[]);

    // Condition
    builder.switch_to_block(cond_block);
    if let Some(cond) = condition {
        let cond_val = self.compile_expr(builder, cond, blocks)?;
        builder.ins().brif(cond_val, body_block, &[], exit_block, &[]);
    } else {
        // 无限循环
        builder.ins().jump(body_block, &[]);
    }

    // Body
    builder.switch_to_block(body_block);
    self.compile_block(builder, body, blocks)?;
    builder.ins().jump(update_block, &[]);

    // Update
    builder.switch_to_block(update_block);
    if let Some(update_stmt) = update {
        self.compile_stmt(builder, update_stmt, blocks)?;
    }
    builder.ins().jump(cond_block, &[]);

    // Exit
    builder.switch_to_block(exit_block);

    // 恢复 break/continue 目标
    blocks.break_target = saved_break;
    blocks.continue_target = saved_continue;

    // Seal blocks
    builder.seal_block(cond_block);
    builder.seal_block(body_block);
    builder.seal_block(update_block);
    builder.seal_block(exit_block);

    Ok(())
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo run -- tests/control_flow/for_loop_control.xin`
Expected: 输出 15 和 5

- [ ] **Step 5: Commit**

```bash
git add crates/xin-codegen/src/cranelift.rs tests/control_flow/for_loop_control.xin
git commit -m "feat(codegen): implement for loop with proper control flow"
```

---

## 里程碑 3: break/continue 实现

### Task 3.1: IR 层添加 break/continue

**Files:**
- Modify: `crates/xin-ir/src/ir.rs`

- [ ] **Step 1: 编写失败测试**

```rust
#[test]
fn test_break_continue_instructions() {
    assert!(matches!(Instruction::Break, Instruction::Break));
    assert!(matches!(Instruction::Continue, Instruction::Continue));
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p xin-ir -- test_break_continue_instructions`
Expected: FAIL - 变体不存在

- [ ] **Step 3: 添加 Break 和 Continue 指令**

```rust
pub enum Instruction {
    // ... existing instructions

    /// Break out of loop
    Break,

    /// Continue to next iteration
    Continue,
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p xin-ir -- test_break_continue_instructions`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/xin-ir/src/ir.rs
git commit -m "feat(ir): add Break and Continue instructions"
```

---

### Task 3.2: Semantic 层循环上下文检查

**Files:**
- Modify: `crates/xin-semantic/src/type_check.rs`
- Modify: `crates/xin-semantic/src/error.rs`

- [ ] **Step 1: 编写失败测试**

```rust
#[test]
fn test_break_outside_loop_error() {
    let result = check_program("break;");
    assert!(matches!(result, Err(SemanticError::BreakOutsideLoop)));
}

#[test]
fn test_continue_outside_loop_error() {
    let result = check_program("continue;");
    assert!(matches!(result, Err(SemanticError::ContinueOutsideLoop)));
}

#[test]
fn test_break_inside_loop_ok() {
    let result = check_program(r#"
        for (var i = 0; i < 10; i = i + 1) {
            if (i == 5) { break; }
        }
    "#);
    assert!(result.is_ok());
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p xin-semantic -- test_break test_continue`
Expected: FAIL - 检查未实现

- [ ] **Step 3: 添加循环深度跟踪**

```rust
pub struct TypeChecker {
    // ... existing fields
    loop_depth: usize,
}

impl TypeChecker {
    pub fn check_stmt(&mut self, stmt: &Stmt) -> Result<Type, SemanticError> {
        match stmt {
            Stmt::For { .. } => {
                self.loop_depth += 1;
                let result = self.check_for_stmt(stmt);
                self.loop_depth -= 1;
                result
            }
            Stmt::Break => {
                if self.loop_depth == 0 {
                    return Err(SemanticError::BreakOutsideLoop);
                }
                Ok(Type::Void)
            }
            Stmt::Continue => {
                if self.loop_depth == 0 {
                    return Err(SemanticError::ContinueOutsideLoop);
                }
                Ok(Type::Void)
            }
            // ...
        }
    }
}
```

- [ ] **Step 4: 添加错误类型**

```rust
pub enum SemanticError {
    // ... existing errors
    BreakOutsideLoop,
    ContinueOutsideLoop,
}
```

- [ ] **Step 5: 运行测试确认通过**

Run: `cargo test -p xin-semantic -- test_break test_continue`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/xin-semantic/src/type_check.rs crates/xin-semantic/src/error.rs
git commit -m "feat(semantic): add loop context check for break/continue"
```

---

### Task 3.3: Codegen 实现 break/continue

**Files:**
- Modify: `crates/xin-codegen/src/cranelift.rs`

- [ ] **Step 1: 编写失败测试**

创建 `tests/control_flow/break_continue.xin`:

```xin
// 测试 break
let sum1 = 0
for (var i = 0; i < 100; i = i + 1) {
    if (i == 5) { break }
    sum1 = sum1 + i
}
println(sum1)  // 期望输出: 10 (0+1+2+3+4)

// 测试 continue
let sum2 = 0
for (var i = 0; i < 5; i = i + 1) {
    if (i == 2) { continue }
    sum2 = sum2 + i
}
println(sum2)  // 期望输出: 8 (0+1+3+4)
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo run -- tests/control_flow/break_continue.xin`
Expected: FAIL - break/continue 未实现

- [ ] **Step 3: 实现 break/continue 代码生成**

```rust
fn compile_break(&self, builder: &mut FunctionBuilder, blocks: &BlockContext) -> Result<(), String> {
    if let Some(break_label) = &blocks.break_target {
        let target = blocks.cranelift_blocks.get(break_label)
            .ok_or_else(|| format!("Break target block '{}' not found", break_label))?;
        builder.ins().jump(*target, &[]);
    }
    Ok(())
}

fn compile_continue(&self, builder: &mut FunctionBuilder, blocks: &BlockContext) -> Result<(), String> {
    if let Some(continue_label) = &blocks.continue_target {
        let target = blocks.cranelift_blocks.get(continue_label)
            .ok_or_else(|| format!("Continue target block '{}' not found", continue_label))?;
        builder.ins().jump(*target, &[]);
    }
    Ok(())
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo run -- tests/control_flow/break_continue.xin`
Expected: 输出 10 和 8

- [ ] **Step 5: Commit**

```bash
git add crates/xin-codegen/src/cranelift.rs tests/control_flow/break_continue.xin
git commit -m "feat(codegen): implement break and continue"
```

---

## 里程碑 4: 函数返回值修复

### Task 4.1: 诊断函数返回问题

**Files:**
- Create: `tests/functions/return_test.xin`

- [ ] **Step 1: 创建诊断测试**

```xin
// tests/functions/return_test.xin

func simple_return() int32 {
    return 42
}

func add(a: int32, b: int32) int32 {
    return a + b
}

func factorial(n: int32) int32 {
    if (n <= 1) { return 1 }
    return n * factorial(n - 1)
}

println(simple_return())     // 期望: 42
println(add(10, 20))         // 期望: 30
println(factorial(5))        // 期望: 120
```

- [ ] **Step 2: 运行测试确认问题**

Run: `cargo run -- tests/functions/return_test.xin`
Expected: 输出不正确，记录问题

- [ ] **Step 3: 打印 IR 诊断**

Run: `cargo run -- --emit-ir tests/functions/return_test.xin`

分析 IR 输出，确认返回值传递问题

---

### Task 4.2: 修复函数返回值

**Files:**
- Modify: `crates/xin-codegen/src/cranelift.rs`

- [ ] **Step 1: 分析问题**

根据 IR 诊断，确定返回值传递问题的根本原因

- [ ] **Step 2: 实现修复**

确保 `Instruction::Return` 和函数调用正确处理返回值

- [ ] **Step 3: 运行测试确认通过**

Run: `cargo run -- tests/functions/return_test.xin`
Expected: 输出 42, 30, 120

- [ ] **Step 4: Commit**

```bash
git add crates/xin-codegen/src/cranelift.rs tests/functions/return_test.xin
git commit -m "fix(codegen): fix function return value passing"
```

---

## 里程碑 5-8 概要

由于篇幅限制，后续里程碑遵循相同的 TDD 模式实现。每个里程碑包含：

1. **测试先行** - 先编写失败测试
2. **最小实现** - 实现使测试通过的最小代码
3. **重构优化** - 在测试保护下优化代码
4. **提交** - 每个功能点单独提交

### 里程碑 5: 类型转换

- Task 5.1: AST 添加类型转换表达式 (失败测试 → 实现 → 通过)
- Task 5.2: Parser 解析类型转换语法 `int32(x)`
- Task 5.3: Semantic 类型转换合法性检查
- Task 5.4: Codegen 生成转换指令
- Task 5.5: 运行时添加字符串转换函数
- Task 5.6: E2E 测试验证

### 里程碑 6: 空安全操作符

- Task 6.1: AST 添加可空类型 `T?` (已存在 `Nullable`)
- Task 6.2: Lexer 添加 `null` 关键字 (已存在 `Null`)
- Task 6.3: Parser 解析 `?.` 和 `??` 操作符 (已存在 Token)
- Task 6.4: Semantic 可空类型检查 (赋值、运算、比较规则)
- Task 6.5: Codegen null 检查和跳转
- Task 6.6: E2E 测试验证

### 里程碑 7: Lambda 表达式

- Task 7.1: AST 添加 Lambda 类型 `func(A) R` (已存在 `Function`)
- Task 7.2: Parser 解析 Lambda 语法 `(a, b) -> a + b`
- Task 7.3: Semantic Lambda 类型推断和捕获分析
- Task 7.4: IR Lambda 表示
- Task 7.5: Codegen 闭包对象生成
- Task 7.6: E2E 测试验证

### 里程碑 8: Map 字面量

- Task 8.1: AST 添加 Map 类型 `map<K, V>` (使用 `Generic`)
- Task 8.2: Parser 解析 Map 语法 `{key: value}`
- Task 8.3: Semantic Map 类型推断
- Task 8.4: 运行时添加 Map 支持 (C hashmap)
- Task 8.5: Codegen Map 操作
- Task 8.6: E2E 测试验证

---

## 执行顺序总结

1. **里程碑 1** (类型系统重构) - 所有后续工作的基础
2. **里程碑 2** (控制流修复) - 让现有测试通过
3. **里程碑 3** (break/continue) - 完善循环
4. **里程碑 4** (函数返回值) - 修复核心功能
5. **里程碑 5-8** - 新增特性

每个任务遵循 TDD 流程：测试 → 失败 → 实现 → 通过 → 提交。