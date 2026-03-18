# 数组（Array）设计文档

**日期**: 2026-03-19

## 1. 概述

数组是 Xin 语言的动态数组类型，支持混合类型元素、嵌套数组、索引访问和可变操作。数组在运行时动态分配在堆上。

## 2. 语法规范

### 2.1 数组字面量

```xin
let a = [1, 2, 3]              // int[]
let b = [1, "a", true]         // object[] (混合类型)
let c = [[1, 2], [3, 4]]       // int[][] (嵌套)
let d = []                      // object[] (空数组默认类型)
let e: string[] = []            // string[] (显式类型)
```

### 2.2 类型注解

```xin
let x: int[] = [1, 2, 3]        // 不可变数组
let y: mut int[] = [1, 2, 3]    // 可变数组
```

### 2.3 索引访问

```xin
let first = a[0]                // 获取元素
let nested = c[0][1]            // 嵌套访问
let dynamic = a[i + 1]          // 动态索引
```

### 2.4 可变数组操作

```xin
let arr: mut int[] = [1, 2, 3]
arr[0] = 10                     // 元素赋值
arr.push(4)                     // 追加元素
arr.pop()                       // 弹出元素
let len = arr.len()             // 获取长度
```

## 3. 类型系统

### 3.1 类型定义

| 类型 | 说明 | 示例 |
|------|------|------|
| `T[]` | 不可变数组 | `int[]`, `string[]` |
| `mut T[]` | 可变数组 | `mut int[]` |
| `object[]` | 混合类型数组 | `[1, "a", true]` |

### 3.2 类型推断规则

1. **统一类型**：所有元素类型一致 → `T[]`
2. **混合类型**：元素类型不一致 → `object[]`
3. **空数组**：无类型注解 → `object[]`
4. **显式注解**：使用注解类型

```xin
let a = [1, 2, 3]           // int[]
let b = [1, "a"]            // object[]
let c = []                  // object[]
let d: string[] = []        // string[]
```

### 3.3 可变性检查

| 操作 | `T[]` (不可变) | `mut T[]` (可变) |
|------|---------------|-----------------|
| 索引读取 | ✓ | ✓ |
| 索引赋值 | ✗ | ✓ |
| push | ✗ | ✓ |
| pop | ✗ | ✓ |
| len | ✓ | ✓ |

## 4. AST 表示

### 4.1 现有类型（已实现）

```rust
// ExprKind
ArrayLiteral(Vec<Expr>),           // 数组字面量
Index { object: Box<Expr>, index: Box<Expr> },  // 索引访问

// Type
Array(Box<Type>),                  // 数组类型
```

### 4.2 新增类型

```rust
// crates/xin-ast/src/ty.rs

pub enum Type {
    // ... 现有类型
    /// object 类型，表示任意类型
    Object,
}
```

### 4.3 类型可变性

在变量声明中处理可变性：

```rust
// crates/xin-ast/src/stmt.rs

pub struct VarDecl {
    pub name: String,
    pub type_annotation: Option<Type>,
    pub value: Option<Expr>,
    pub mutable: bool,  // 已存在
}
```

数组类型本身不携带可变性，可变性由变量声明决定。

## 5. 实现细节

### 5.1 Lexer 层

已有 `LBracket` (`[`)、`RBracket` (`]`)、`Mut` 等 token，无需修改。

### 5.2 Parser 层

**类型解析**：

```rust
// crates/xin-parser/src/parser.rs

fn parse_type(&mut self) -> Result<Type, ParserError> {
    let mut ty = self.parse_primary_type()?;

    // 数组后缀: T[]
    while self.match_kind(TokenKind::LBracket) {
        self.consume(TokenKind::RBracket, "expected ']'")?;
        ty = Type::Array(Box::new(ty));
    }

    Ok(ty)
}
```

**方法调用解析**（已支持 `MethodCall`）：

```xin
arr.push(1)  // MethodCall { object: arr, method: "push", args: [1] }
```

### 5.3 Semantic 层

**类型检查**：

```rust
// crates/xin-semantic/src/type_check.rs

fn check_array_literal(&mut self, elements: &[Expr]) -> Result<Type, SemanticError> {
    if elements.is_empty() {
        return Ok(Type::Array(Box::new(Type::Object)));
    }

    let types: Vec<Type> = elements.iter().map(|e| self.check_expr(e)).collect::<Result<_, _>>()?;

    // 检查是否所有类型一致
    let first_type = &types[0];
    let all_same = types.iter().all(|t| t == first_type);

    if all_same {
        Ok(Type::Array(Box::new(first_type.clone())))
    } else {
        Ok(Type::Array(Box::new(Type::Object)))
    }
}

fn check_index_access(&mut self, object: &Expr, index: &Expr) -> Result<Type, SemanticError> {
    let obj_type = self.check_expr(object)?;
    let idx_type = self.check_expr(index)?;

    // 索引必须是 int
    if idx_type != Type::Int {
        return Err(SemanticError::TypeMismatch {
            expected: Type::Int,
            actual: idx_type,
        });
    }

    // 对象必须是数组
    match obj_type {
        Type::Array(elem_type) => Ok(*elem_type),
        _ => Err(SemanticError::NotIndexable(obj_type)),
    }
}
```

**可变性检查**：

```rust
fn check_method_call(&mut self, object: &Expr, method: &str, is_mutable: bool) -> Result<Type, SemanticError> {
    match method {
        "push" | "pop" if !is_mutable => {
            Err(SemanticError::ImmutableArrayModification(method.to_string()))
        }
        _ => Ok(/* ... */)
    }
}
```

### 5.4 IR 层

**新增指令**：

```rust
// crates/xin-ir/src/ir.rs

pub enum Instruction {
    // ... 现有指令

    /// 创建数组: %result = array_new type, capacity
    ArrayNew {
        result: Value,
        elem_type: IRType,
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

**IR 生成**：

```rust
// crates/xin-ir/src/builder.rs

fn build_array_literal(&mut self, elements: &[Expr]) -> Option<Value> {
    let capacity = elements.len();
    let result = self.new_temp();

    // 创建数组
    self.emit(Instruction::ArrayNew {
        result: result.clone(),
        elem_type: IRType::Object, // 简化处理
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
```

### 5.5 Codegen 层

```rust
// crates/xin-codegen/src/aot.rs

fn compile_instruction(&mut self, inst: &Instruction, ...) -> Result<(), CodegenError> {
    match inst {
        Instruction::ArrayNew { result, elem_type, capacity } => {
            let arr_ptr = self.call_runtime("xin_array_new", &[
                &capacity.to_string(),
                &elem_type.to_type_id().to_string(),
            ])?;
            self.store_variable(builder, result, arr_ptr, variables, var_counter);
        }
        Instruction::ArrayGet { result, array, index } => {
            let arr = self.load_variable(builder, array, variables)?;
            let idx = self.load_variable(builder, index, variables)?;
            let val = self.call_runtime("xin_array_get", &[arr, idx])?;
            self.store_variable(builder, result, val, variables, var_counter);
        }
        // ... 其他指令
    }
}
```

### 5.6 Runtime 层

```c
// runtime/runtime.c

#include <stdlib.h>
#include <stdio.h>
#include <string.h>

// 元素类型标记
#define XIN_TYPE_INT    0
#define XIN_TYPE_FLOAT  1
#define XIN_TYPE_BOOL   2
#define XIN_TYPE_STRING 3
#define XIN_TYPE_OBJECT 4

// 数组结构
typedef struct {
    void** data;
    int64_t length;
    int64_t capacity;
    int8_t elem_type;
} xin_array;

// 创建数组
xin_array* xin_array_new(int64_t capacity, int8_t elem_type) {
    xin_array* arr = malloc(sizeof(xin_array));
    if (!arr) return NULL;

    arr->data = calloc(capacity, sizeof(void*));
    if (!arr->data) {
        free(arr);
        return NULL;
    }

    arr->length = 0;
    arr->capacity = capacity;
    arr->elem_type = elem_type;
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

// 设置元素
void xin_array_set(xin_array* arr, int64_t index, void* value) {
    if (index < 0 || index >= arr->length) {
        fprintf(stderr, "ArrayIndexOutOfBoundsError: index %lld out of bounds for length %lld\n",
                (long long)index, (long long)arr->length);
        exit(1);
    }
    arr->data[index] = value;
}

// 追加元素
void xin_array_push(xin_array* arr, void* value) {
    if (arr->length >= arr->capacity) {
        // 扩容
        int64_t new_capacity = arr->capacity == 0 ? 4 : arr->capacity * 2;
        void** new_data = realloc(arr->data, new_capacity * sizeof(void*));
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

## 6. 错误处理

### 6.1 错误类型

```rust
// crates/xin-semantic/src/error.rs

pub enum SemanticError {
    // ... 现有错误

    /// 类型不可索引
    NotIndexable(Type),
    /// 不可变数组修改
    ImmutableArrayModification(String),
    /// 类型不匹配（数组元素）
    ArrayElementTypeMismatch { expected: Type, actual: Type, index: usize },
}
```

### 6.2 错误信息示例

```
error: type `int` is not indexable
  --> main.xin:3:10
   |
3  |     let x = 10[0]
   |            ^^^^^ type `int` does not support indexing

error: cannot modify immutable array
  --> main.xin:5:5
   |
5  |     arr[0] = 1
   |     ^^^^^^^^^^ array `arr` is immutable
   |
   = help: declare as `mut int[]` to allow modification

error: array element type mismatch
  --> main.xin:2:20
   |
2  |     let arr: int[] = [1, "a"]
   |                       ^^^^^^ expected `int`, found `string` at index 1

runtime error: ArrayIndexOutOfBoundsError: index 5 out of bounds for length 3
```

## 7. 测试用例

### 7.1 E2E 测试

```xin
// tests/arrays/basic.xin
func main() {
    let arr = [1, 2, 3]
    println(arr[0])   // 1
    println(arr[1])   // 2
}

// tests/arrays/nested.xin
func main() {
    let matrix = [[1, 2], [3, 4]]
    println(matrix[0][1])  // 2
}

// tests/arrays/mixed.xin
func main() {
    let mixed = [1, "hello", true]
    println(mixed[1])  // hello
}

// tests/arrays/mutable.xin
func main() {
    let arr: mut int[] = [1, 2, 3]
    arr[0] = 10
    arr.push(4)
    println(arr[0])   // 10
    println(arr[3])   // 4
    println(arr.len()) // 4
}

// tests/arrays/empty.xin
func main() {
    let empty: int[] = []
    println(empty.len())  // 0
}
```

### 7.2 运行时错误测试

```xin
// tests/arrays/out_of_bounds.xin
func main() {
    let arr = [1, 2, 3]
    println(arr[10])  // runtime panic
}
```

## 8. 实现步骤

1. **AST**：添加 `Type::Object`
2. **Parser**：解析 `mut T[]` 类型语法
3. **Semantic**：数组类型推断、可变性检查
4. **IR**：添加数组相关指令
5. **Codegen**：生成运行时函数调用
6. **Runtime**：实现数组操作函数
7. **Tests**：添加 E2E 测试

## 9. 未来扩展

- 数组切片：`arr[1:3]`
- 数组推导式：`[x * 2 for x in arr]`
- 多维数组语法糖：`int[3][3]`
- 数组复制方法：`arr.clone()`