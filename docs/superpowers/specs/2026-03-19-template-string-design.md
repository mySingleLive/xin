# 字符串模板设计文档

**日期**: 2026-03-19

## 1. 概述

字符串模板（Template String）是 Xin 语言的一种新语法，允许在字符串中嵌入表达式。使用反引号括起来，支持花括号 `{}` 包裹的嵌入表达式。

## 2. 语法规范

### 2.1 基本语法

```
`hello`                    // 纯文本
`name is {name}`           // 嵌入变量
`value = {a + n * 10}`     // 嵌入表达式
`nested {"{"}`             // 表达式中嵌套字符串
`complex {foo("}")}`       // 复杂表达式，智能匹配括号
```

### 2.2 转义规则

| 转义序列 | 结果 |
|---------|------|
| `` \` `` | `` ` `` |
| `\\` | `\` |
| `\{` | `{` |
| `\}` | `}` |
| `\n` | 换行 |
| `\t` | 制表符 |

### 2.3 表达式解析规则

嵌入表达式使用智能解析，支持：

1. **括号匹配**：`{foo(1, 2)}` 正确解析
2. **字符串字面量**：`{"hello}"}` 中 `}` 不结束表达式
3. **嵌套模板**：`{bar(`inner`)}` 正确解析
4. **任意复杂表达式**：与外部 Xin 表达式等价

### 2.4 示例

```xin
// 基本用法
let name = "Alice"
let greeting = `Hello, {name}!`        // "Hello, Alice!"

// 表达式嵌入
let a = 10
let b = 20
let result = `sum = {a + b}`            // "sum = 30"

// 复杂表达式
func add(x: int, y: int) int -> x + y
let s = `result = {add(1, 2) * 10}`     // "result = 30"

// 多行模板
let multi = `line1
line2
line3`

// 转义字符
let escaped = `brace: \{, backtick: \`` // "brace: {, backtick: `"

// 智能匹配
let nested = `json: {"key": "value}"}`  // 正确解析
```

## 3. 类型规则

### 3.1 返回类型

模板字符串返回 `string` 类型。

### 3.2 表达式类型约束

嵌入表达式必须是可字符串化类型：

| 类型 | 转换方式 |
|-----|---------|
| `int` | 调用 `int_to_string` |
| `float` | 调用 `float_to_string` |
| `bool` | 调用 `bool_to_string` |
| `string` | 直接拼接 |

### 3.3 类型检查示例

```xin
let a: int = 10
let b: float = 3.14
let c: bool = true
let d: string = "hello"

// 合法
let s1 = `{a}`           // int → string
let s2 = `{b}`           // float → string
let s3 = `{c}`           // bool → string
let s4 = `{d}`           // string → string
let s5 = `{a + 1}`       // 表达式结果 int → string

// 非法
let user = User { name: "Bob" }
let s6 = `{user}`        // 错误：User 不能转换为 string
```

## 4. AST 表示

### 4.1 新增类型

```rust
// crates/xin-ast/src/expr.rs

/// 模板字符串部分
#[derive(Debug, Clone)]
pub enum TemplatePart {
    /// 纯文本
    Text(String),
    /// 嵌入表达式
    Expr(Box<Expr>),
}

// ExprKind 新增变体
pub enum ExprKind {
    // ... 现有类型

    /// 字符串模板: `hello {name}`
    TemplateLiteral(Vec<TemplatePart>),
}
```

### 4.2 AST 示例

```
`Hello, {name}! You have {count} messages.`
```

转换为：

```rust
ExprKind::TemplateLiteral(vec![
    TemplatePart::Text("Hello, "),
    TemplatePart::Expr(Box::new(Expr {
        kind: ExprKind::Ident("name"),
        ...
    })),
    TemplatePart::Text("! You have "),
    TemplatePart::Expr(Box::new(Expr {
        kind: ExprKind::Ident("count"),
        ...
    })),
    TemplatePart::Text(" messages."),
])
```

## 5. 实现细节

### 5.1 Lexer 层

**Token 新增**：

```rust
// crates/xin-ast/src/token.rs

pub enum TokenKind {
    // ... 现有类型
    TemplateString,   // 字符串模板
}
```

**词法分析逻辑**：

```rust
// crates/xin-lexer/src/lexer.rs

fn template_string(&mut self, start_line: usize, start_col: usize) -> Result<Token, LexerError> {
    let mut value = String::new();

    while !self.is_at_end() && self.peek() != '`' {
        let ch = self.advance();
        if ch == '\\' {
            // 处理转义，保留原样供 Parser 解析
            value.push(ch);
            if let Some(escaped) = self.peek_next() {
                value.push(self.advance());
            }
        } else {
            value.push(ch);
        }
    }

    if self.is_at_end() {
        return Err(LexerError::UnterminatedTemplate);
    }

    self.advance(); // closing `
    Ok(Token::new(TokenKind::TemplateString, value, start_line, start_col))
}
```

### 5.2 Parser 层

**模板解析**：

```rust
// crates/xin-parser/src/parser.rs

fn parse_template_literal(&mut self, raw: &str, span: SourceSpan) -> Result<Expr, ParserError> {
    let mut parts = Vec::new();
    let mut chars = raw.chars().peekable();
    let mut text = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                // 处理转义
                if let Some(escaped) = chars.next() {
                    match escaped {
                        '`' | '\\' | '{' | '}' => text.push(escaped),
                        'n' => text.push('\n'),
                        't' => text.push('\t'),
                        'r' => text.push('\r'),
                        _ => return Err(ParserError::InvalidEscape(escaped, span)),
                    }
                }
            }
            '{' => {
                // 保存前面的文本
                if !text.is_empty() {
                    parts.push(TemplatePart::Text(text.clone()));
                    text.clear();
                }
                // 解析嵌入表达式
                let expr = self.parse_template_expr(&mut chars, span)?;
                parts.push(TemplatePart::Expr(Box::new(expr)));
            }
            _ => text.push(ch),
        }
    }

    // 保存剩余文本
    if !text.is_empty() {
        parts.push(TemplatePart::Text(text));
    }

    Ok(Expr::new(ExprKind::TemplateLiteral(parts), span))
}
```

**智能表达式解析**：

```rust
fn parse_template_expr(&mut self, chars: &mut Peekable<Chars>, span: SourceSpan) -> Result<Expr, ParserError> {
    let mut expr_chars = Vec::new();
    let mut brace_count = 1;
    let mut string_delim: Option<char> = None;

    while brace_count > 0 {
        let ch = chars.next().ok_or(ParserError::UnclosedTemplateExpr(span))?;

        match (ch, string_delim) {
            // 进入字符串
            ('"' | '`', None) => {
                string_delim = Some(ch);
                expr_chars.push(ch);
            }
            // 退出字符串
            (c, Some(d)) if c == d => {
                string_delim = None;
                expr_chars.push(ch);
            }
            // 字符串内转义
            ('\\', Some(_)) => {
                expr_chars.push(ch);
                if let Some(next) = chars.next() {
                    expr_chars.push(next);
                }
            }
            // 字符串外的花括号
            ('{', None) => {
                brace_count += 1;
                expr_chars.push(ch);
            }
            ('}', None) => {
                brace_count -= 1;
                if brace_count > 0 {
                    expr_chars.push(ch);
                }
            }
            // 其他字符
            _ => expr_chars.push(ch),
        }
    }

    // 将收集的字符作为表达式解析
    let expr_str: String = expr_chars.into_iter().collect();
    self.parse_expr_from_str(&expr_str, span)
}
```

### 5.3 语义分析层

```rust
// crates/xin-semantic/src/type_check.rs

fn check_template_literal(&mut self, parts: &[TemplatePart], span: SourceSpan) -> Result<Type, SemanticError> {
    for part in parts {
        if let TemplatePart::Expr(expr) = part {
            let ty = self.check_expr(expr)?;
            if !self.is_stringifiable(&ty) {
                return Err(SemanticError::CannotConvertToString {
                    ty,
                    span: expr.span,
                });
            }
        }
    }
    Ok(Type::String)
}

fn is_stringifiable(&self, ty: &Type) -> bool {
    matches!(ty, Type::Int | Type::Float | Type::Bool | Type::String)
}
```

### 5.4 IR 层

**新增指令**：

```rust
// crates/xin-ir/src/ir.rs

pub enum Instruction {
    // ... 现有指令

    /// 类型转字符串: %result = to_string value
    ToString {
        result: Value,
        value: Value,
        from_type: IRType,
    },
}
```

**IR 生成**：

```rust
// crates/xin-ir/src/builder.rs

fn lower_template_literal(&mut self, parts: &[TemplatePart]) -> Result<Value, IRError> {
    let mut result: Option<Value> = None;

    for part in parts {
        match part {
            TemplatePart::Text(text) => {
                let idx = self.module.add_string(text);
                let text_val = self.new_temp();
                self.emit(Instruction::StringConst {
                    result: text_val.clone(),
                    string_index: idx,
                });
                result = Some(self.concat_strings(result, text_val)?);
            }
            TemplatePart::Expr(expr) => {
                let expr_val = self.lower_expr(expr)?;
                let ty = self.get_expr_type(expr);
                let str_val = self.to_string(expr_val, ty)?;
                result = Some(self.concat_strings(result, str_val)?);
            }
        }
    }

    Ok(result.unwrap_or_else(|| self.empty_string()))
}

fn to_string(&mut self, value: Value, ty: IRType) -> Result<Value, IRError> {
    if ty == IRType::String {
        return Ok(value);
    }

    let result = self.new_temp();
    self.emit(Instruction::ToString {
        result: result.clone(),
        value,
        from_type: ty,
    });
    Ok(result)
}
```

### 5.5 Codegen 层

```rust
// crates/xin-codegen/src/cranelift.rs

// 在 compile_instruction 中添加
Instruction::ToString { result, value, from_type } => {
    let val = self.load_variable(builder, value, variables)?;
    let result_val = match from_type {
        IRType::I64 => self.call_runtime("int_to_string", &[val]),
        IRType::F64 => self.call_runtime("float_to_string", &[val]),
        IRType::Bool => self.call_runtime("bool_to_string", &[val]),
        _ => val,
    };
    self.store_variable(builder, result, result_val, variables, var_counter);
}
```

### 5.6 Runtime 层

```c
// runtime/runtime.c

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

char* int_to_string(int64_t n) {
    char* buf = malloc(32);
    snprintf(buf, 32, "%ld", n);
    return buf;
}

char* float_to_string(double d) {
    char* buf = malloc(64);
    snprintf(buf, 64, "%g", d);
    return buf;
}

char* bool_to_string(int8_t b) {
    return b ? strdup("true") : strdup("false");
}
```

## 6. 错误处理

### 6.1 错误类型

```rust
// crates/xin-lexer/src/error.rs
pub enum LexerError {
    // ... 现有错误
    UnterminatedTemplate,
}

// crates/xin-parser/src/error.rs
pub enum ParserError {
    // ... 现有错误
    InvalidEscape(char, SourceSpan),
    UnclosedTemplateExpr(SourceSpan),
}

// crates/xin-semantic/src/error.rs
pub enum SemanticError {
    // ... 现有错误
    CannotConvertToString { ty: Type, span: SourceSpan },
}
```

### 6.2 错误信息示例

```
error: unterminated template string
  --> main.xin:3:15
   |
3  |     let s = `hello
   |               ^^^^^ unterminated template string

error: unclosed template expression
  --> main.xin:4:20
   |
4  |     let s = `value = {1 + 2`
   |                    ^^^^^^^^ expected `}` to close expression

error: cannot convert type `User` to string
  --> main.xin:5:25
   |
5  |     let s = `user: {user}`
   |                         ^ type `User` cannot be converted to string
   |
   = help: implement a `toString()` method for `User`
```

## 7. 测试用例

### 7.1 词法测试

```rust
#[test]
fn test_template_string_simple() {
    let mut lexer = Lexer::new("`hello`");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::TemplateString);
    assert_eq!(tokens[0].text, "hello");
}

#[test]
fn test_template_string_with_expr() {
    let mut lexer = Lexer::new("`name is {name}`");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::TemplateString);
    assert_eq!(tokens[0].text, "name is {name}");
}
```

### 7.2 语法测试

```rust
#[test]
fn test_parse_template_simple() {
    let ast = parse("`hello`").unwrap();
    assert!(matches!(ast, ExprKind::TemplateLiteral(parts) if parts.len() == 1));
}

#[test]
fn test_parse_template_with_expr() {
    let ast = parse("`a = {a}`").unwrap();
    match ast {
        ExprKind::TemplateLiteral(parts) => {
            assert_eq!(parts.len(), 3);
            assert!(matches!(&parts[0], TemplatePart::Text(t) if t == "a = "));
            assert!(matches!(&parts[1], TemplatePart::Expr(_)));
            assert!(matches!(&parts[2], TemplatePart::Text(t) if t == ""));
        }
        _ => panic!("expected TemplateLiteral"),
    }
}
```

### 7.3 语义测试

```rust
#[test]
fn test_template_type_check() {
    let result = type_check("`value = {1 + 2}`");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::String);
}

#[test]
fn test_template_invalid_type() {
    let result = type_check("let u = User{}; `{u}`");
    assert!(result.is_err());
}
```

### 7.4 E2E 测试

```xin
// tests/template_basic.xin
func main() {
    let name = "World"
    let greeting = `Hello, {name}!`
    println(greeting)  // 输出: Hello, World!
}

// tests/template_expr.xin
func main() {
    let a = 10
    let b = 20
    println(`sum = {a + b}`)  // 输出: sum = 30
}

// tests/template_escape.xin
func main() {
    println(`backtick: \``)   // 输出: backtick: `
    println(`brace: \{}`)     // 输出: brace: {}
}
```

## 8. 实现步骤

1. **Lexer**：添加 `TemplateString` token 和反引号解析
2. **Parser**：实现模板解析和智能表达式匹配
3. **AST**：添加 `TemplateLiteral` 和 `TemplatePart` 类型
4. **Semantic**：实现类型检查
5. **IR**：添加 `ToString` 指令，实现 IR 生成
6. **Codegen**：实现运行时调用
7. **Runtime**：添加类型转换函数
8. **Tests**：添加单元测试和 E2E 测试

## 9. 未来扩展

- 支持自定义类型的 `toString()` 方法
- 支持格式化表达式 `{value:.2f}`
- 支持原始模板字符串（不处理转义）