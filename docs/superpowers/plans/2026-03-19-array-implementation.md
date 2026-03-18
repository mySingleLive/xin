# 数组（Array）实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 Xin 语言实现动态数组支持，包括数组字面量、索引访问、可变操作和运行时支持。

**Architecture:** 采用分层实现：AST 层添加 Object 类型 → Parser 层支持 `mut T[]` 语法 → Semantic 层实现类型推断和可变性检查 → IR 层添加数组指令 → Codegen 层生成运行时调用 → Runtime 层实现数组操作函数。

**Tech Stack:** Rust (编译器), C (运行时), Cranelift (代码生成)

---

## 文件结构

### 需要修改的文件

| 文件 | 职责 |
|------|------|
| `crates/xin-ast/src/ty.rs` | 添加 `Type::Object` 类型 |
| `crates/xin-parser/src/parser.rs` | 解析 `mut T[]` 类型语法，设置 `object_mutable` |
| `crates/xin-semantic/src/type_check.rs` | 数组类型推断（空数组 → object[]），可变性检查 |
| `crates/xin-semantic/src/error.rs` | 添加数组相关错误类型 |
| `crates/xin-semantic/src/symbol.rs` | 确保 Symbol 有 `object_mutable` 字段 |
| `crates/xin-ir/src/ir.rs` | 添加数组相关 IR 指令 |
| `crates/xin-ir/src/builder.rs` | 生成数组 IR 代码 |
| `crates/xin-codegen/src/aot.rs` | 编译数组指令为运行时调用 |
| `runtime/runtime.c` | 实现数组运行时函数 |

### 需要创建的测试文件

| 文件 | 职责 |
|------|------|
| `tests/arrays/basic.xin` | 基本数组操作测试 |
| `tests/arrays/nested.xin` | 嵌套数组测试 |
| `tests/arrays/mutable.xin` | 可变数组测试 |

---

### Task 1: 添加 Object 类型到 AST

**Files:**
- Modify: `crates/xin-ast/src/ty.rs:1-81`

- [ ] **Step 1: 添加 Object 类型变体**

在 `Type` 枚举中添加 `Object` 变体：

```rust
// crates/xin-ast/src/ty.rs

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// int
    Int,
    /// float
    Float,
    /// bool
    Bool,
    /// string
    String,
    /// void
    Void,
    /// object 类型，表示任意类型的运行时值
    Object,
    /// User-defined type name
    Named(String),
    // ... 其余不变
}
```

- [ ] **Step 2: 更新 Display trait 实现**

```rust
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Void => write!(f, "void"),
            Type::Object => write!(f, "object"),
            // ... 其余不变
        }
    }
}
```

- [ ] **Step 3: 验证编译通过**

Run: `cargo build`
Expected: 编译成功，无错误

- [ ] **Step 4: Commit**

```bash
git add crates/xin-ast/src/ty.rs
git commit -m "feat(ast): add Object type for mixed-type arrays"
```

---

### Task 2: 更新 Parser 支持 mut T[] 语法

**Files:**
- Modify: `crates/xin-parser/src/parser.rs:280-360`

- [ ] **Step 1: 定位变量声明解析代码**

阅读 `parse_var_decl` 函数，理解当前如何解析类型注解。

- [ ] **Step 2: 修改类型注解解析逻辑**

在解析类型注解时，检查 `mut` 关键字：

```rust
// crates/xin-parser/src/parser.rs
// 在 parse_var_decl 或类似函数中

// 解析类型注解
let (type_annotation, object_mutable) = if self.match_kind(TokenKind::Colon) {
    // 检查 mut 前缀
    let has_mut = self.match_kind(TokenKind::Mut);
    let ty = self.parse_type()?;
    (Some(ty), has_mut)
} else {
    (None, false)
};
```

- [ ] **Step 3: 更新 VarDecl 构造**

确保 `object_mutable` 字段正确设置：

```rust
Ok(Stmt::new(
    StmtKind::VarDecl(VarDecl {
        name,
        mutable,  // let/var 决定
        type_annotation,
        value,
        object_mutable,  // 类型注解中的 mut 决定
    }),
    span,
))
```

- [ ] **Step 4: 验证编译通过**

Run: `cargo build`
Expected: 编译成功

- [ ] **Step 5: Commit**

```bash
git add crates/xin-parser/src/parser.rs
git commit -m "feat(parser): support mut T[] syntax for mutable arrays"
```

---

### Task 3: 更新 Semantic 层数组类型推断

**Files:**
- Modify: `crates/xin-semantic/src/type_check.rs:777-794`
- Modify: `crates/xin-semantic/src/error.rs`

- [ ] **Step 1: 修改空数组类型推断**

将空数组从 `Type::Void` 改为 `Type::Object`：

```rust
// crates/xin-semantic/src/type_check.rs

ExprKind::ArrayLiteral(elements) => {
    if elements.is_empty() {
        // 空数组默认为 object[]
        return Ok(Type::Array(Box::new(Type::Object)));
    }

    let elem_type = self.check_expr(&elements[0])?;
    for elem in &elements[1..] {
        let t = self.check_expr(elem)?;
        if !self.types_compatible(&elem_type, &t) {
            // 混合类型 → object[]
            // 如果不是所有元素类型一致，返回 object[]
            return Ok(Type::Array(Box::new(Type::Object)));
        }
    }

    Ok(Type::Array(Box::new(elem_type)))
}
```

- [ ] **Step 2: 添加数组相关错误类型**

```rust
// crates/xin-semantic/src/error.rs

pub enum SemanticError {
    // ... 现有错误

    /// 类型不可索引
    NotIndexable { ty: Type, span: SourceSpan },

    /// 不可变数组修改
    ImmutableArrayModification { method: String, span: SourceSpan },

    /// 数组元素类型不匹配（用于类型注解与字面量不匹配）
    ArrayElementTypeMismatch {
        expected: Type,
        actual: Type,
        index: usize,
        span: SourceSpan,
    },
}
```

- [ ] **Step 3: 实现索引访问类型检查**

查找现有的 `ExprKind::Index` 处理，确保：
- 索引表达式类型为 `int`
- 对象类型为数组
- 返回元素类型

- [ ] **Step 4: 验证编译通过**

Run: `cargo build`
Expected: 编译成功

- [ ] **Step 5: Commit**

```bash
git add crates/xin-semantic/src/type_check.rs crates/xin-semantic/src/error.rs
git commit -m "feat(semantic): array type inference with object[] for mixed/empty arrays"
```

---

### Task 4: 实现可变性检查

**Files:**
- Modify: `crates/xin-semantic/src/type_check.rs`
- Modify: `crates/xin-semantic/src/symbol.rs`

- [ ] **Step 1: 检查 Symbol 结构体**

确保 `Symbol` 有 `object_mutable` 字段。如果没有，添加：

```rust
// crates/xin-semantic/src/symbol.rs

pub struct Symbol {
    pub name: String,
    pub ty: Type,
    pub mutable: bool,        // 变量可重新赋值
    pub object_mutable: bool, // 对象可修改
    // ...
}
```

- [ ] **Step 2: 实现方法调用可变性检查**

在处理 `ExprKind::MethodCall` 时，检查 push/pop 操作：

```rust
// crates/xin-semantic/src/type_check.rs

ExprKind::MethodCall { object, method, args } => {
    // 检查可变性
    if let ExprKind::Ident(name) = &object.kind {
        if let Some(symbol) = self.scopes.lookup(name) {
            match method.as_str() {
                "push" | "pop" if !symbol.object_mutable => {
                    return Err(SemanticError::ImmutableArrayModification {
                        method: method.clone(),
                        span: expr.span.clone(),
                    });
                }
                _ => {}
            }
        }
    }

    // 类型检查...
}
```

- [ ] **Step 3: 实现索引赋值可变性检查**

在处理赋值语句时，检查索引赋值：

```rust
// 在赋值检查中
StmtKind::Expr(Expr { kind: ExprKind::Assign { target, value }, .. }) => {
    if let ExprKind::Index { object, .. } = &target.kind {
        if let ExprKind::Ident(name) = &object.kind {
            if let Some(symbol) = self.scopes.lookup(name) {
                if !symbol.object_mutable {
                    return Err(SemanticError::ImmutableArrayModification {
                        method: "index assignment".to_string(),
                        span: target.span.clone(),
                    });
                }
            }
        }
    }
    // ... 继续类型检查
}
```

- [ ] **Step 4: 验证编译通过**

Run: `cargo build`
Expected: 编译成功

- [ ] **Step 5: Commit**

```bash
git add crates/xin-semantic/src/type_check.rs crates/xin-semantic/src/symbol.rs
git commit -m "feat(semantic): mutability checks for array operations"
```

---

### Task 5: 添加数组 IR 指令

**Files:**
- Modify: `crates/xin-ir/src/ir.rs:56-132`
- Modify: `crates/xin-ir/src/ir.rs:16-38`

- [ ] **Step 1: 添加 IRType::Object**

```rust
// crates/xin-ir/src/ir.rs

pub enum IRType {
    I64,
    F64,
    Bool,
    String,
    Void,
    Ptr(String),
    Object, // 新增
}
```

- [ ] **Step 2: 更新 IRType Display**

```rust
impl fmt::Display for IRType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IRType::Object => write!(f, "object"),
            // ... 其余不变
        }
    }
}
```

- [ ] **Step 3: 添加数组 IR 指令**

```rust
// crates/xin-ir/src/ir.rs

pub enum Instruction {
    // ... 现有指令

    /// 创建数组: %result = array_new capacity
    ArrayNew {
        result: Value,
        capacity: usize,
    },

    /// 获取元素: %result = array_get array, index
    ArrayGet {
        result: Value,
        array: Value,
        index: Value,
    },

    /// 设置元素: array_set array, index, value
    ArraySet {
        array: Value,
        index: Value,
        value: Value,
    },

    /// 追加元素: array_push array, value
    ArrayPush {
        array: Value,
        value: Value,
    },

    /// 弹出元素: %result = array_pop array
    ArrayPop {
        result: Value,
        array: Value,
    },

    /// 获取长度: %result = array_len array
    ArrayLen {
        result: Value,
        array: Value,
    },
}
```

- [ ] **Step 4: 验证编译通过**

Run: `cargo build`
Expected: 编译成功

- [ ] **Step 5: Commit**

```bash
git add crates/xin-ir/src/ir.rs
git commit -m "feat(ir): add array instructions"
```

---

### Task 6: 实现 IR Builder 数组代码生成

**Files:**
- Modify: `crates/xin-ir/src/builder.rs:515-519`

- [ ] **Step 1: 实现数组字面量 IR 生成**

```rust
// crates/xin-ir/src/builder.rs

// 在 build_expr 的 match 中，替换 _ => None 之前添加：

ExprKind::ArrayLiteral(elements) => {
    let capacity = elements.len();
    let result = self.new_temp();

    // 创建数组
    self.emit(Instruction::ArrayNew {
        result: result.clone(),
        capacity,
    });

    // 填充元素
    for (i, elem) in elements.iter().enumerate() {
        let elem_val = self.build_expr(elem)?;
        let index = self.new_temp();
        self.emit(Instruction::Const {
            result: index.clone(),
            value: i.to_string(),
            ty: IRType::I64,
        });
        self.emit(Instruction::ArraySet {
            array: result.clone(),
            index,
            value: elem_val,
        });
    }

    Some(result)
}

ExprKind::Index { object, index } => {
    let obj_val = self.build_expr(object)?;
    let idx_val = self.build_expr(index)?;
    let result = self.new_temp();

    self.emit(Instruction::ArrayGet {
        result: result.clone(),
        array: obj_val,
        index: idx_val,
    });

    Some(result)
}
```

- [ ] **Step 2: 实现 len() 方法调用 IR 生成**

```rust
ExprKind::MethodCall { object, method, args } => {
    match method.as_str() {
        "len" => {
            let obj_val = self.build_expr(object)?;
            let result = self.new_temp();
            self.emit(Instruction::ArrayLen {
                result: result.clone(),
                array: obj_val,
            });
            Some(result)
        }
        "push" => {
            let obj_val = self.build_expr(object)?;
            let arg_val = self.build_expr(&args[0])?;
            self.emit(Instruction::ArrayPush {
                array: obj_val,
                value: arg_val,
            });
            None
        }
        "pop" => {
            let obj_val = self.build_expr(object)?;
            let result = self.new_temp();
            self.emit(Instruction::ArrayPop {
                result: result.clone(),
                array: obj_val,
            });
            Some(result)
        }
        _ => {
            // 其他方法调用处理...
            None
        }
    }
}
```

- [ ] **Step 3: 更新 convert_type 支持 Object**

```rust
fn convert_type(&self, ty: &Type) -> IRType {
    match ty {
        Type::Object => IRType::Object,
        // ... 其余不变
    }
}
```

- [ ] **Step 4: 验证编译通过**

Run: `cargo build`
Expected: 编译成功

- [ ] **Step 5: Commit**

```bash
git add crates/xin-ir/src/builder.rs
git commit -m "feat(ir): implement array IR generation"
```

---

### Task 7: 实现运行时数组函数

**Files:**
- Modify: `runtime/runtime.c`

- [ ] **Step 1: 添加数组结构体定义**

```c
// runtime/runtime.c
// 在文件末尾添加

// ========== Array Runtime ==========

// 数组结构
typedef struct {
    void** data;        // 元素指针数组
    int64_t length;     // 当前长度
    int64_t capacity;   // 容量
} xin_array;

// 创建数组
xin_array* xin_array_new(int64_t capacity) {
    xin_array* arr = (xin_array*)malloc(sizeof(xin_array));
    if (!arr) return NULL;

    arr->data = (void**)calloc(capacity > 0 ? capacity : 4, sizeof(void*));
    if (!arr->data && capacity > 0) {
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
```

- [ ] **Step 2: 验证编译通过**

Run: `gcc -c runtime/runtime.c -o runtime/runtime.o`
Expected: 编译成功，无错误

- [ ] **Step 3: Commit**

```bash
git add runtime/runtime.c
git commit -m "feat(runtime): implement array operations"
```

---

### Task 8: 实现 Codegen 数组指令编译

**Files:**
- Modify: `crates/xin-codegen/src/aot.rs`

- [ ] **Step 1: 添加外部函数声明**

在编译模块初始化时，声明数组运行时函数：

```rust
// crates/xin-codegen/src/aot.rs

fn declare_extern_functions(&mut self) {
    // ... 现有声明

    // 数组函数
    self.module
        .declare_function("xin_array_new", cranelift::module::Linkage::Import, ...)
        .expect("failed to declare xin_array_new");

    self.module
        .declare_function("xin_array_get", cranelift::module::Linkage::Import, ...)
        .expect("failed to declare xin_array_get");

    self.module
        .declare_function("xin_array_set", cranelift::module::Linkage::Import, ...)
        .expect("failed to declare xin_array_set");

    self.module
        .declare_function("xin_array_push", cranelift::module::Linkage::Import, ...)
        .expect("failed to declare xin_array_push");

    self.module
        .declare_function("xin_array_pop", cranelift::module::Linkage::Import, ...)
        .expect("failed to declare xin_array_pop");

    self.module
        .declare_function("xin_array_len", cranelift::module::Linkage::Import, ...)
        .expect("failed to declare xin_array_len");
}
```

- [ ] **Step 2: 实现指令编译**

```rust
fn compile_instruction(&mut self, inst: &Instruction, ...) -> Result<(), CodegenError> {
    match inst {
        // ... 现有指令

        Instruction::ArrayNew { result, capacity } => {
            let capacity_val = self.builder.ins().iconst(
                self.target_config.pointer_type(),
                *capacity as i64
            );
            let arr_ptr = self.call_runtime("xin_array_new", &[capacity_val])?;
            self.store_variable(result, arr_ptr);
        }

        Instruction::ArrayGet { result, array, index } => {
            let arr = self.load_variable(array)?;
            let idx = self.load_variable(index)?;
            let val = self.call_runtime("xin_array_get", &[arr, idx])?;
            self.store_variable(result, val);
        }

        Instruction::ArraySet { array, index, value } => {
            let arr = self.load_variable(array)?;
            let idx = self.load_variable(index)?;
            let val = self.load_variable(value)?;
            self.call_runtime_void("xin_array_set", &[arr, idx, val])?;
        }

        Instruction::ArrayPush { array, value } => {
            let arr = self.load_variable(array)?;
            let val = self.load_variable(value)?;
            self.call_runtime_void("xin_array_push", &[arr, val])?;
        }

        Instruction::ArrayPop { result, array } => {
            let arr = self.load_variable(array)?;
            let val = self.call_runtime("xin_array_pop", &[arr])?;
            self.store_variable(result, val);
        }

        Instruction::ArrayLen { result, array } => {
            let arr = self.load_variable(array)?;
            let len = self.call_runtime("xin_array_len", &[arr])?;
            self.store_variable(result, len);
        }
    }
}
```

- [ ] **Step 3: 验证编译通过**

Run: `cargo build`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add crates/xin-codegen/src/aot.rs
git commit -m "feat(codegen): compile array instructions to runtime calls"
```

---

### Task 9: 添加 E2E 测试

**Files:**
- Create: `tests/arrays/basic.xin`
- Create: `tests/arrays/nested.xin`
- Create: `tests/arrays/mutable.xin`

- [ ] **Step 1: 创建基本数组测试**

```xin
// tests/arrays/basic.xin
func main() {
    let arr = [1, 2, 3]
    println(arr[0])
    println(arr[1])
    println(arr[2])
}
```

- [ ] **Step 2: 运行基本测试**

Run: `cargo run -- compile tests/arrays/basic.xin -o /tmp/basic && /tmp/basic`
Expected: 输出 `1`, `2`, `3`

- [ ] **Step 3: 创建嵌套数组测试**

```xin
// tests/arrays/nested.xin
func main() {
    let matrix = [[1, 2], [3, 4]]
    println(matrix[0][0])
    println(matrix[0][1])
    println(matrix[1][0])
    println(matrix[1][1])
}
```

- [ ] **Step 4: 运行嵌套测试**

Run: `cargo run -- compile tests/arrays/nested.xin -o /tmp/nested && /tmp/nested`
Expected: 输出 `1`, `2`, `3`, `4`

- [ ] **Step 5: 创建可变数组测试**

```xin
// tests/arrays/mutable.xin
func main() {
    let arr: mut int[] = [1, 2, 3]
    arr[0] = 10
    arr.push(4)
    println(arr[0])
    println(arr[3])
    println(arr.len())
}
```

- [ ] **Step 6: 运行可变测试**

Run: `cargo run -- compile tests/arrays/mutable.xin -o /tmp/mutable && /tmp/mutable`
Expected: 输出 `10`, `4`, `4`

- [ ] **Step 7: Commit**

```bash
git add tests/arrays/
git commit -m "test: add array e2e tests"
```

---

### Task 10: 最终验证和集成

- [ ] **Step 1: 运行所有测试**

Run: `cargo test`
Expected: 所有测试通过

- [ ] **Step 2: 运行完整编译器测试套件**

Run: `./scripts/run_e2e_tests.sh` (如果存在) 或手动运行所有 tests/ 目录下的测试

Expected: 所有测试通过

- [ ] **Step 3: 最终 Commit**

```bash
git add -A
git commit -m "feat: complete array implementation with mutability support"
```

---

## 实现顺序依赖

```
Task 1 (AST) ──────┬──> Task 3 (Semantic) ──> Task 4 (Mutability)
                   │
Task 2 (Parser) ───┘

Task 5 (IR) ───────> Task 6 (IR Builder)

Task 7 (Runtime) ──> Task 8 (Codegen)

Task 1-8 全部完成 ──> Task 9 (Tests) ──> Task 10 (Integration)
```

## 关键注意事项

1. **类型推断**：空数组 `[]` 应推断为 `object[]`，而非 `void[]`
2. **可变性两级模型**：
   - `let`/`var` → `mutable` 字段（变量重新赋值）
   - `mut T[]` → `object_mutable` 字段（数组元素修改）
3. **运行时安全**：所有数组操作都有边界检查，越界时 panic
4. **混合类型数组**：当元素类型不一致时，自动推断为 `object[]`