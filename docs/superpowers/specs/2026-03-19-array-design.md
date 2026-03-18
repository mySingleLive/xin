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

### 3.3 类型兼容性规则

**基本规则**：类型必须完全匹配，不支持协变或逆变。

```xin
let a: int[] = [1, 2, 3]
let b: object[] = a    // ✗ 错误：int[] 不能赋值给 object[]
let c: object[] = [1, "a"]  // ✓ 正确：字面量推断为 object[]
```

**可变性规则**：

| 源类型 | 目标类型 | 是否允许 |
|--------|---------|---------|
| `T[]` | `T[]` | ✓ |
| `mut T[]` | `T[]` | ✓ (丢弃可变性) |
| `T[]` | `mut T[]` | ✗ |
| `mut T[]` | `mut T[]` | ✓ |

**嵌套可变性**：可变性只影响顶层数组，不影响嵌套数组。

```xin
let matrix: mut int[][] = [[1, 2], [3, 4]]
matrix[0] = [5, 6]       // ✓ 外层可变
matrix[0][0] = 10        // ✗ 内层不可变 (int[][] 非 mut int[][])
```

### 3.4 可变性检查

| 操作 | `T[]` (不可变) | `mut T[]` (可变) |
|------|---------------|-----------------|
| 索引读取 | ✓ | ✓ |
| 索引赋值 | ✗ | ✓ |
| push | ✗ | ✓ |
| pop | ✗ | ✓ |
| len | ✓ | ✓ |

### 3.5 数组方法签名

| 方法 | 签名 | 返回类型 | 说明 |
|------|------|---------|------|
| `push` | `arr.push(value: T)` | `void` | 追加元素 |
| `pop` | `arr.pop()` | `T` | 弹出最后一个元素 |
| `len` | `arr.len()` | `int` | 返回数组长度 |

方法在类型系统中作为内置方法处理，不需要显式定义 trait。

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
    /// object 类型，表示任意类型的运行时值
    /// 用于混合类型数组的元素类型
    Object,
}
```

**Object 类型语义**：
- `Object` 是顶级类型，可以接受任何值
- 主要用于 `object[]` 数组的元素类型
- 运行时通过 `elem_type` 标记区分实际类型

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

通过作用域查找变量的可变性标记：

```rust
fn check_method_call(&mut self, object: &Expr, method: &str) -> Result<Type, SemanticError> {
    // 从表达式中提取变量名
    if let ExprKind::Ident(name) = &object.kind {
        // 从作用域查找变量的可变性
        let symbol = self.scopes.lookup(name)?;
        let is_mutable = symbol.is_mutable();

        match method {
            "push" | "pop" if !is_mutable => {
                return Err(SemanticError::ImmutableArrayModification {
                    method: method.to_string(),
                    span: object.span.clone(),
                });
            }
            _ => {}
        }
    }

    // 执行类型检查
    let obj_type = self.check_expr(object)?;
    // ...
}
```

**索引赋值可变性检查**：

```rust
fn check_assignment(&mut self, target: &Expr, value: &Expr) -> Result<Type, SemanticError> {
    match &target.kind {
        ExprKind::Index { object, index: _ } => {
            // 检查 object 是否可变
            if let ExprKind::Ident(name) = &object.kind {
                let symbol = self.scopes.lookup(name)?;
                if !symbol.is_mutable() {
                    return Err(SemanticError::ImmutableArrayModification {
                        method: "index assignment".to_string(),
                        span: target.span.clone(),
                    });
                }
            }
        }
        // ... 其他情况
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

> **设计说明**：当前实现使用 `void**` 统一存储所有类型，简化实现。未来可优化为根据元素类型使用不同的存储策略（如 `int[]` 使用连续内存存储）。

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
// 注意：当前使用 void** 存储，每个元素都是指针
// 对于 int[] 等基本类型数组，这会增加内存开销
// 未来优化：使用 union 或类型特化存储
typedef struct {
    void** data;        // 元素指针数组
    int64_t length;     // 当前长度
    int64_t capacity;   // 容量
    int8_t elem_type;   // 元素类型标记
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

## 6. 内存管理

### 6.1 生命周期

数组在堆上动态分配，生命周期遵循以下规则：

1. **创建**：数组字面量或 `xin_array_new` 创建时分配
2. **使用**：函数内创建的数组在函数执行期间有效
3. **释放**：当前实现不自动释放，依赖程序退出时操作系统回收

### 6.2 所有权语义（简化版本）

当前实现采用简化策略：
- 数组元素存储指针/值拷贝
- 不追踪所有权转移
- 不自动释放内存

未来可扩展：
- 添加 `drop(arr)` 显式释放
- 实现引用计数或 GC
- 所有权系统

## 7. 错误处理

### 7.1 错误类型与代码

```rust
// crates/xin-semantic/src/error.rs

pub enum SemanticError {
    // ... 现有错误

    /// S004: 类型不可索引
    NotIndexable { ty: Type, span: SourceSpan },

    /// S005: 不可变数组修改
    ImmutableArrayModification { method: String, span: SourceSpan },

    /// S006: 数组元素类型不匹配
    ArrayElementTypeMismatch {
        expected: Type,
        actual: Type,
        index: usize,
        span: SourceSpan,
    },
}
```

### 7.2 边界情况处理

| 情况 | 处理方式 | 检测阶段 |
|------|---------|---------|
| 负数索引 | 运行时检查，panic | 运行时 |
| 非整数索引 | 编译时报类型错误 | 语义分析 |
| 嵌套索引赋值 `matrix[0][1] = 5` | 检查每层可变性 | 语义分析 |
| `pop()` 空数组 | 运行时 panic | 运行时 |
| 空数组 `len()` | 返回 0 | 正常 |

### 7.3 错误信息示例

```
error[S004]: type `int` is not indexable
  --> main.xin:3:10
   |
3  |     let x = 10[0]
   |            ^^^^^ type `int` does not support indexing

error[S005]: cannot modify immutable array
  --> main.xin:5:5
   |
5  |     arr[0] = 1
   |     ^^^^^^^^^^ array `arr` is immutable
   |
   = help: declare as `mut int[]` to allow modification

error[S006]: array element type mismatch
  --> main.xin:2:20
   |
2  |     let arr: int[] = [1, "a"]
   |                       ^^^^^^ expected `int`, found `string` at index 1

runtime error: ArrayIndexOutOfBoundsError: index 5 out of bounds for length 3
```

## 8. 测试用例

### 8.1 E2E 测试

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

### 8.2 运行时错误测试

```xin
// tests/arrays/out_of_bounds.xin
func main() {
    let arr = [1, 2, 3]
    println(arr[10])  // runtime panic
}
```

## 9. 实现步骤

1. **AST**：添加 `Type::Object`
2. **Parser**：解析 `mut T[]` 类型语法
3. **Semantic**：数组类型推断、可变性检查
4. **IR**：添加数组相关指令
5. **Codegen**：生成运行时函数调用
6. **Runtime**：实现数组操作函数
7. **Tests**：添加 E2E 测试

## 10. 未来扩展

- 数组切片：`arr[1:3]`
- 数组推导式：`[x * 2 for x in arr]`
- 多维数组语法糖：`int[3][3]`
- 数组复制方法：`arr.clone()`