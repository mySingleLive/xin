# Template String Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add template string support to Xin language, enabling string interpolation with `{expression}` syntax inside backtick-delimited strings.

**Architecture:** Template strings flow through all compiler stages: Lexer recognizes backticks and emits raw template content, Parser parses embedded expressions with smart bracket matching, AST adds `TemplateLiteral` node, Semantic checks type validity, IR generates string concatenation, Codegen emits runtime calls.

**Tech Stack:** Rust, Cranelift, C runtime

---

## File Structure

| File | Responsibility |
|------|---------------|
| `crates/xin-ast/src/token.rs` | Add `TemplateString` token kind |
| `crates/xin-ast/src/expr.rs` | Add `TemplateLiteral` and `TemplatePart` types |
| `crates/xin-lexer/src/lexer.rs` | Parse backtick-delimited template strings |
| `crates/xin-lexer/src/error.rs` | Add `UnterminatedTemplate` error |
| `crates/xin-parser/src/parser.rs` | Parse template expressions with bracket matching |
| `crates/xin-parser/src/error.rs` | Add template parsing errors |
| `crates/xin-semantic/src/type_check.rs` | Type check template expressions |
| `crates/xin-semantic/src/error.rs` | Add `CannotConvertToString` error |
| `crates/xin-ir/src/ir.rs` | Add `ToString` instruction |
| `crates/xin-ir/src/builder.rs` | Generate IR for template literals |
| `crates/xin-codegen/src/cranelift.rs` | Codegen for `ToString` instruction |
| `runtime/runtime.c` | Add type-to-string conversion functions |
| `tests/templates/*.xin` | E2E test cases |

---

## Task 1: Add AST Types for Template Strings

**Files:**
- Modify: `crates/xin-ast/src/token.rs`
- Modify: `crates/xin-ast/src/expr.rs`

- [ ] **Step 1: Add `TemplateString` token kind**

In `crates/xin-ast/src/token.rs`, add to `TokenKind` enum (after `StringLiteral`):

```rust
    // Literals
    IntLiteral,
    FloatLiteral,
    StringLiteral,
    TemplateString,  // Add this line
    BoolLiteral,
```

And add to the `Display` impl:

```rust
            TokenKind::StringLiteral => write!(f, "string literal"),
            TokenKind::TemplateString => write!(f, "template string"),
            TokenKind::BoolLiteral => write!(f, "boolean literal"),
```

- [ ] **Step 2: Add `TemplatePart` and `TemplateLiteral` types**

In `crates/xin-ast/src/expr.rs`, add after `UnaryOp` definition:

```rust
/// Template string part
#[derive(Debug, Clone)]
pub enum TemplatePart {
    /// Plain text
    Text(String),
    /// Embedded expression
    Expr(Box<Expr>),
}
```

And add to `ExprKind` enum (after `StringLiteral`):

```rust
    /// String literal: "hello"
    StringLiteral(String),
    /// Template string: `hello {name}`
    TemplateLiteral(Vec<TemplatePart>),
    /// Boolean literal: true, false
    BoolLiteral(bool),
```

- [ ] **Step 3: Verify compilation**

Run: `cargo build --package xin-ast`
Expected: Compiles successfully with no errors

- [ ] **Step 4: Commit**

```bash
git add crates/xin-ast/src/token.rs crates/xin-ast/src/expr.rs
git commit -m "feat(ast): add TemplateString token and TemplateLiteral expression types"
```

---

## Task 2: Add Lexer Support for Template Strings

**Files:**
- Modify: `crates/xin-lexer/src/error.rs`
- Modify: `crates/xin-lexer/src/lexer.rs`

- [ ] **Step 1: Add `UnterminatedTemplate` error**

In `crates/xin-lexer/src/error.rs`, add to `LexerError` enum:

```rust
#[derive(Debug, Clone)]
pub enum LexerError {
    UnexpectedChar(char),
    UnterminatedString,
    UnterminatedTemplate,  // Add this
    InvalidEscape(char),
    InvalidNumber(String),
    UnterminatedComment,
}
```

And add to `Display` impl:

```rust
            LexerError::UnterminatedString => write!(f, "unterminated string literal"),
            LexerError::UnterminatedTemplate => write!(f, "unterminated template string"),
            LexerError::InvalidEscape(c) => write!(f, "invalid escape character: '{}'", c),
```

- [ ] **Step 2: Add template string parsing in lexer**

In `crates/xin-lexer/src/lexer.rs`, in `next_token` method, add handling for backtick after the `'"'` case:

```rust
            // String literal
            '"' => self.string_literal(start_line, start_col),

            // Template string literal
            '`' => self.template_string(start_line, start_col),
```

Then add the `template_string` method after `string_literal`:

```rust
    fn template_string(&mut self, start_line: usize, start_col: usize) -> Result<Token, LexerError> {
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != '`' {
            let ch = self.advance();
            if ch == '\\' {
                // Keep escape sequences for parser to handle
                value.push(ch);
                if !self.is_at_end() {
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

- [ ] **Step 3: Write unit test for template string lexer**

Add to the test module in `crates/xin-lexer/src/lexer.rs`:

```rust
    #[test]
    fn test_tokenize_template_string() {
        let mut lexer = Lexer::new("`hello`");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2); // template + EOF
        assert_eq!(tokens[0].kind, TokenKind::TemplateString);
        assert_eq!(tokens[0].text, "hello");
    }

    #[test]
    fn test_tokenize_template_with_expr() {
        let mut lexer = Lexer::new("`name is {name}`");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::TemplateString);
        assert_eq!(tokens[0].text, "name is {name}");
    }

    #[test]
    fn test_tokenize_template_with_escape() {
        let mut lexer = Lexer::new("`backtick: \\``");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::TemplateString);
        assert_eq!(tokens[0].text, "backtick: \\`");
    }
```

- [ ] **Step 4: Run tests to verify lexer works**

Run: `cargo test --package xin-lexer`
Expected: All tests pass including new template tests

- [ ] **Step 5: Commit**

```bash
git add crates/xin-lexer/src/error.rs crates/xin-lexer/src/lexer.rs
git commit -m "feat(lexer): add template string tokenization"
```

---

## Task 3: Add Parser Support for Template Strings

**Files:**
- Modify: `crates/xin-parser/src/error.rs`
- Modify: `crates/xin-parser/src/parser.rs`

- [ ] **Step 1: Add template parsing errors**

In `crates/xin-parser/src/error.rs`, add to `ParserError` enum:

```rust
pub enum ParserError {
    // ... existing errors
    InvalidEscape(char),
    UnclosedTemplateExpr,
}
```

And add to `Display` impl:

```rust
            ParserError::InvalidEscape(c) => write!(f, "invalid escape character: '{}'", c),
            ParserError::UnclosedTemplateExpr => write!(f, "unclosed template expression"),
```

- [ ] **Step 2: Add template parsing in parser**

In `crates/xin-parser/src/parser.rs`, find the `parse_primary` method and add handling for `TemplateString`:

```rust
            TokenKind::StringLiteral => {
                let text = self.advance().text.clone();
                Ok(Expr::new(ExprKind::StringLiteral(text), span))
            }
            TokenKind::TemplateString => {
                let text = self.advance().text.clone();
                self.parse_template_literal(&text, span)
            }
```

Then add the template parsing methods. First add imports at the top:

```rust
use std::iter::Peekable;
use std::str::Chars;
```

Then add these methods after `parse_primary`:

```rust
    fn parse_template_literal(&mut self, raw: &str, span: SourceSpan) -> Result<Expr, ParserError> {
        let mut parts = Vec::new();
        let mut chars = raw.chars().peekable();
        let mut text = String::new();

        while let Some(ch) = chars.next() {
            match ch {
                '\\' => {
                    if let Some(escaped) = chars.next() {
                        match escaped {
                            '`' | '\\' | '{' | '}' => text.push(escaped),
                            'n' => text.push('\n'),
                            't' => text.push('\t'),
                            'r' => text.push('\r'),
                            _ => {
                                return Err(ParserError::InvalidEscape(escaped));
                            }
                        }
                    }
                }
                '{' => {
                    if !text.is_empty() {
                        parts.push(TemplatePart::Text(text.clone()));
                        text.clear();
                    }
                    let expr = self.parse_template_expr(&mut chars)?;
                    parts.push(TemplatePart::Expr(Box::new(expr)));
                }
                _ => text.push(ch),
            }
        }

        if !text.is_empty() {
            parts.push(TemplatePart::Text(text));
        }

        Ok(Expr::new(ExprKind::TemplateLiteral(parts), span))
    }

    fn parse_template_expr(&mut self, chars: &mut Peekable<Chars>) -> Result<Expr, ParserError> {
        let mut expr_chars = String::new();
        let mut brace_count = 1;
        let mut string_delim: Option<char> = None;

        while brace_count > 0 {
            let ch = chars.next().ok_or(ParserError::UnclosedTemplateExpr)?;

            match (ch, string_delim) {
                ('"' | '\'', None) => {
                    string_delim = Some(ch);
                    expr_chars.push(ch);
                }
                (c, Some(d)) if c == d => {
                    string_delim = None;
                    expr_chars.push(ch);
                }
                ('\\', Some(_)) => {
                    expr_chars.push(ch);
                    if let Some(next) = chars.next() {
                        expr_chars.push(next);
                    }
                }
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
                _ => expr_chars.push(ch),
            }
        }

        // Parse the collected expression string
        self.parse_expr_from_str(&expr_chars)
    }

    fn parse_expr_from_str(&mut self, expr_str: &str) -> Result<Expr, ParserError> {
        // Create a temporary lexer and parser for the expression
        let mut temp_lexer = Lexer::new(expr_str);
        let temp_tokens = temp_lexer.tokenize().map_err(|e| {
            ParserError::LexerError(e.to_string())
        })?;

        // Save current state
        let original_tokens = std::mem::replace(&mut self.tokens, temp_tokens);
        let original_current = self.current;
        self.current = 0;

        // Parse expression
        let result = self.parse_expression();

        // Restore state
        self.tokens = original_tokens;
        self.current = original_current;

        result
    }
```

Add the import for `Lexer` at the top:

```rust
use xin_lexer::Lexer;
```

- [ ] **Step 3: Run parser tests**

Run: `cargo test --package xin-parser`
Expected: Tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/xin-parser/src/error.rs crates/xin-parser/src/parser.rs
git commit -m "feat(parser): add template string parsing with smart bracket matching"
```

---

## Task 4: Add Semantic Type Checking for Templates

**Files:**
- Modify: `crates/xin-semantic/src/error.rs`
- Modify: `crates/xin-semantic/src/type_check.rs`

- [ ] **Step 1: Add `CannotConvertToString` error**

In `crates/xin-semantic/src/error.rs`, add to `SemanticError` enum:

```rust
pub enum SemanticError {
    // ... existing errors
    CannotConvertToString { ty: Type, span: SourceSpan },
}
```

And add to `Display` impl:

```rust
            SemanticError::CannotConvertToString { ty, .. } => {
                write!(f, "cannot convert type '{}' to string", ty)
            }
```

- [ ] **Step 2: Add template type checking**

In `crates/xin-semantic/src/type_check.rs`, find the `check_expr` method and add handling for `TemplateLiteral`:

```rust
            ExprKind::TemplateLiteral(parts) => {
                self.check_template_literal(parts, &expr.span)
            }
```

Then add the helper methods:

```rust
    fn check_template_literal(&mut self, parts: &[TemplatePart], span: &SourceSpan) -> Result<Type, SemanticError> {
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

Add the import at the top:

```rust
use xin_ast::TemplatePart;
```

- [ ] **Step 3: Run semantic tests**

Run: `cargo test --package xin-semantic`
Expected: Tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/xin-semantic/src/error.rs crates/xin-semantic/src/type_check.rs
git commit -m "feat(semantic): add type checking for template strings"
```

---

## Task 5: Add IR Support for Template Strings

**Files:**
- Modify: `crates/xin-ir/src/ir.rs`
- Modify: `crates/xin-ir/src/builder.rs`

- [ ] **Step 1: Add `ToString` instruction**

In `crates/xin-ir/src/ir.rs`, add to `Instruction` enum (after `StringConcat`):

```rust
    /// Convert to string: %result = to_string value
    ToString {
        result: Value,
        value: Value,
        from_type: IRType,
    },
```

- [ ] **Step 2: Add IR generation for templates**

In `crates/xin-ir/src/builder.rs`, add import:

```rust
use xin_ast::TemplatePart;
```

In `build_expr` method, add handling for `TemplateLiteral`:

```rust
            ExprKind::TemplateLiteral(parts) => {
                self.build_template_literal(parts)
            }
```

Then add the helper methods after `build_expr`:

```rust
    fn build_template_literal(&mut self, parts: &[TemplatePart]) -> Option<Value> {
        let mut result: Option<Value> = None;

        for part in parts {
            match part {
                TemplatePart::Text(text) => {
                    let string_index = self.module.add_string(text);
                    let text_val = self.new_temp();
                    self.emit(Instruction::StringConst {
                        result: text_val.clone(),
                        string_index,
                    });
                    result = Some(self.concat_strings(result, text_val));
                }
                TemplatePart::Expr(expr) => {
                    let expr_val = self.build_expr(expr)?;
                    let expr_type = self.get_expr_type_with_vars(expr);
                    let str_val = self.convert_to_string(expr_val, expr_type);
                    result = Some(self.concat_strings(result, str_val));
                }
            }
        }

        result.or_else(|| {
            // Empty template
            let string_index = self.module.add_string("");
            let result = self.new_temp();
            self.emit(Instruction::StringConst {
                result: result.clone(),
                string_index,
            });
            Some(result)
        })
    }

    fn convert_to_string(&mut self, value: Value, ty: Option<Type>) -> Value {
        match ty {
            Some(Type::String) => value,
            Some(Type::Int) => {
                let result = self.new_temp();
                self.emit(Instruction::ToString {
                    result: result.clone(),
                    value,
                    from_type: IRType::I64,
                });
                self.declare_extern_if_needed(
                    "xin_int_to_str",
                    vec![IRType::I64],
                    Some(IRType::Ptr("char".to_string())),
                );
                result
            }
            Some(Type::Float) => {
                let result = self.new_temp();
                self.emit(Instruction::ToString {
                    result: result.clone(),
                    value,
                    from_type: IRType::F64,
                });
                self.declare_extern_if_needed(
                    "xin_float_to_str",
                    vec![IRType::F64],
                    Some(IRType::Ptr("char".to_string())),
                );
                result
            }
            Some(Type::Bool) => {
                let result = self.new_temp();
                self.emit(Instruction::ToString {
                    result: result.clone(),
                    value,
                    from_type: IRType::Bool,
                });
                self.declare_extern_if_needed(
                    "xin_bool_to_str",
                    vec![IRType::Bool],
                    Some(IRType::Ptr("char".to_string())),
                );
                result
            }
            _ => value,
        }
    }

    fn concat_strings(&mut self, left: Option<Value>, right: Value) -> Value {
        match left {
            None => right,
            Some(left_val) => {
                let result = self.new_temp();
                self.emit(Instruction::StringConcat {
                    result: result.clone(),
                    left: left_val,
                    left_type: ConcatType::String,
                    right,
                    right_type: ConcatType::String,
                });
                self.declare_extern_if_needed(
                    "xin_str_concat_ss",
                    vec![IRType::Ptr("char".to_string()), IRType::Ptr("char".to_string())],
                    Some(IRType::Ptr("char".to_string())),
                );
                result
            }
        }
    }
```

- [ ] **Step 3: Run IR tests**

Run: `cargo test --package xin-ir`
Expected: Tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/xin-ir/src/ir.rs crates/xin-ir/src/builder.rs
git commit -m "feat(ir): add ToString instruction and template literal IR generation"
```

---

## Task 6: Add Codegen Support for ToString

**Files:**
- Modify: `crates/xin-codegen/src/cranelift.rs`

- [ ] **Step 1: Add codegen for ToString instruction**

In `crates/xin-codegen/src/cranelift.rs`, find `compile_instruction` method and add handling for `ToString`:

```rust
            Instruction::ToString { result, value, from_type } => {
                let val = self.load_variable(builder, value, variables)?;
                let func_name = match from_type {
                    IRType::I64 => "xin_int_to_str",
                    IRType::F64 => "xin_float_to_str",
                    IRType::Bool => "xin_bool_to_str",
                    _ => "xin_int_to_str",
                };
                let result_val = self.call_external(builder, func_name, &[val], pointer_type)?;
                self.store_variable(builder, result, result_val, variables, var_counter);
            }
```

Add the helper method if not already present:

```rust
    fn call_external(
        &self,
        builder: &mut FunctionBuilder,
        name: &str,
        args: &[Value],
        pointer_type: Type,
    ) -> Result<Value, String> {
        let mut sig = self.module.make_signature();
        for _ in args {
            sig.params.push(AbiParam::new(pointer_type));
        }
        sig.returns.push(AbiParam::new(pointer_type));

        let func_id = self.module
            .declare_function(name, Linkage::Import, &sig)
            .map_err(|e| format!("Failed to declare external function: {}", e))?;

        let func_ref = builder.import_function(func_id);
        let call = builder.ins().call(func_ref, args);
        Ok(builder.inst_results(call)[0])
    }
```

- [ ] **Step 2: Build codegen**

Run: `cargo build --package xin-codegen`
Expected: Compiles successfully

- [ ] **Step 3: Commit**

```bash
git add crates/xin-codegen/src/cranelift.rs
git commit -m "feat(codegen): add codegen for ToString instruction"
```

---

## Task 7: Add Runtime Functions

**Files:**
- Modify: `runtime/runtime.c`

- [ ] **Step 1: Add type-to-string conversion functions**

In `runtime/runtime.c`, add after existing functions:

```c
// Convert integer to string
char* xin_int_to_str(int64_t n) {
    char* buf = malloc(32);
    if (buf == NULL) return NULL;
    snprintf(buf, 32, "%ld", n);
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
```

- [ ] **Step 2: Build runtime**

Run: `cd runtime && make clean && make`
Expected: Compiles successfully

- [ ] **Step 3: Commit**

```bash
git add runtime/runtime.c
git commit -m "feat(runtime): add type-to-string conversion functions"
```

---

## Task 8: Add E2E Tests

**Files:**
- Create: `tests/templates/basic.xin`
- Create: `tests/templates/basic.expected`
- Create: `tests/templates/expressions.xin`
- Create: `tests/templates/expressions.expected`
- Create: `tests/templates/escape.xin`
- Create: `tests/templates/escape.expected`

- [ ] **Step 1: Create basic template test**

Create `tests/templates/basic.xin`:

```xin
// Test basic template strings

func main() {
    let name = "World"
    println(`Hello, {name}!`)

    let count = 42
    println(`Count: {count}`)

    let price = 19.99
    println(`Price: {price}`)

    let active = true
    println(`Active: {active}`)
}
```

Create `tests/templates/basic.expected`:

```
Hello, World!
Count: 42
Price: 19.99
Active: true
```

- [ ] **Step 2: Create expressions test**

Create `tests/templates/expressions.xin`:

```xin
// Test template expressions

func add(a: int, b: int) int {
    return a + b
}

func main() {
    let a = 10
    let b = 20

    println(`Sum: {a + b}`)
    println(`Product: {a * b}`)
    println(`Function: {add(5, 3)}`)
    println(`Complex: {add(a, b) * 2}`)
}
```

Create `tests/templates/expressions.expected`:

```
Sum: 30
Product: 200
Function: 8
Complex: 60
```

- [ ] **Step 3: Create escape test**

Create `tests/templates/escape.xin`:

```xin
// Test template escape sequences

func main() {
    println(`Backtick: \``)
    println(`Brace: \{ \}`)
    println(`Backslash: \\`)
    println(`Newline:\nSecond line`)
    println(`Tab:\there`)
}
```

Create `tests/templates/escape.expected`:

```
Backtick: `
Brace: { }
Backslash: \
Newline:
Second line
Tab:	here
```

- [ ] **Step 4: Update test runner**

Modify `tests/run_tests.sh` to include templates in `PHASE1_DIRS`:

```bash
PHASE1_DIRS=("basic" "strings" "operators" "templates")
```

- [ ] **Step 5: Run e2e tests**

Run: `cd tests && ./run_tests.sh templates`
Expected: All tests pass

- [ ] **Step 6: Commit**

```bash
git add tests/templates/ tests/run_tests.sh
git commit -m "test: add e2e tests for template strings"
```

---

## Task 9: Integration Testing

- [ ] **Step 1: Run all tests**

Run: `cd tests && ./run_tests.sh --all`
Expected: All tests pass

- [ ] **Step 2: Test edge cases**

Create a temporary test file to verify edge cases:

```bash
cat > /tmp/test_template.xin << 'EOF'
func main() {
    // Empty template
    println(``)

    // Template with only text
    println(`hello world`)

    // Template with only expression
    let x = 123
    println(`{x}`)

    // Nested braces in expression
    println(`Calc: {1 + (2 * 3)}`)
}
EOF
cargo run -- compile /tmp/test_template.xin -o /tmp/test_template && /tmp/test_template
```

Expected output:
```

hello world
123
Calc: 7
```

- [ ] **Step 3: Final commit**

```bash
git add -A
git commit -m "feat: complete template string implementation"
```

---

## Summary

| Task | Description | Status |
|------|-------------|--------|
| 1 | Add AST types | ⏳ |
| 2 | Lexer support | ⏳ |
| 3 | Parser support | ⏳ |
| 4 | Semantic type checking | ⏳ |
| 5 | IR generation | ⏳ |
| 6 | Codegen support | ⏳ |
| 7 | Runtime functions | ⏳ |
| 8 | E2E tests | ⏳ |
| 9 | Integration testing | ⏳ |