# Xin AOT 编译器设计文档

**日期**: 2026-03-15

## 1. 概述

### 1.1 目标

实现 Xin 语言的 AOT（Ahead-of-Time）编译器，能够将 Xin 源代码编译为独立的可执行文件。

### 1.2 验收标准

能够成功编译并运行以下程序：

```xin
// hello.xin
func main() {
    println("Hello, World!")
}
```

```bash
xin compile hello.xin -o hello
./hello
# 输出: Hello, World!
```

## 2. 架构设计

### 2.1 编译流程

```
hello.xin
    ↓
┌─────────────────────────────────────────────┐
│  现有编译流程（保持不变）                      │
│  Lexer → Parser → TypeChecker → IRBuilder   │
└─────────────────────────────────────────────┘
    ↓
    IR Module
    ↓
┌─────────────────────────────────────────────┐
│  新增：AOT 代码生成                          │
│  ObjectModule → 生成 hello.o                │
└─────────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────────┐
│  新增：链接阶段                              │
│  调用 cc 链接 hello.o + runtime.c → hello   │
└─────────────────────────────────────────────┘
    ↓
hello (可执行文件)
```

### 2.2 组件职责

| 组件 | 文件 | 职责 |
|-----|------|------|
| AOT CodeGenerator | `crates/xin-codegen/src/aot.rs` | 使用 Cranelift ObjectModule 生成 .o 文件 |
| Runtime | `runtime/runtime.c` | 提供内置函数（xin_print_int 等） |
| Linker | `src/linker.rs` | 调用系统 C 编译器进行链接 |
| Compiler | `src/compiler.rs` | 编排整个编译流程 |

## 3. 详细设计

### 3.1 AOT CodeGenerator

基于 Cranelift 的 `ObjectModule` 实现，与现有的 JIT 模式 `JITModule` 并行：

```rust
// crates/xin-codegen/src/aot.rs
pub struct AOTCodeGenerator {
    module: ObjectModule,
}

impl AOTCodeGenerator {
    pub fn new() -> Result<Self, String> {
        // 使用 ObjectModule 而非 JITModule
    }

    pub fn compile(&mut self, module: &IRModule) -> Result<(), String> {
        // 编译所有函数
    }

    pub fn emit_object(&self) -> Result<Vec<u8>, String> {
        // 生成对象文件字节
    }
}
```

### 3.2 Runtime（C 运行时）

提供内置函数的实现，通过 C 标准库实现 I/O：

```c
// runtime/runtime.c
#include <stdio.h>

// 整数打印
void xin_print_int(long long n) {
    printf("%lld", n);
}

// 浮点数打印
void xin_print_float(double n) {
    printf("%g", n);
}

// 布尔值打印
void xin_print_bool(int b) {
    printf("%s", b ? "true" : "false");
}

// 字符串打印
void xin_print_str(const char* s) {
    printf("%s", s);
}

// 换行
void xin_println() {
    printf("\n");
}
```

### 3.3 Linker

封装系统 C 编译器的调用：

```rust
// src/linker.rs
pub struct Linker {
    c_compiler: String,  // "cc" 或 "gcc" 或 "clang"
}

impl Linker {
    pub fn new() -> Self {
        // 检测系统可用的 C 编译器
    }

    pub fn link(&self, obj_path: &Path, runtime_path: &Path, output: &Path) -> Result<(), String> {
        // 执行: cc obj_path runtime_path -o output
    }
}
```

### 3.4 Compiler 改造

现有编译器使用 JIT 模式，需要改造为 AOT 模式：

```rust
// src/compiler.rs
pub struct Compiler {
    emit_ir: bool,
    output: Option<PathBuf>,
}

impl Compiler {
    pub fn compile(&self, input: &Path) -> anyhow::Result<()> {
        // 1. 现有流程：Lexing → Parsing → Type Checking → IR Generation

        // 2. 新增：AOT 代码生成
        let mut codegen = AOTCodeGenerator::new()?;
        codegen.compile(&ir_module)?;
        let obj_bytes = codegen.emit_object()?;

        // 3. 新增：写入对象文件
        let obj_path = output.with_extension("o");
        std::fs::write(&obj_path, &obj_bytes)?;

        // 4. 新增：链接
        let linker = Linker::new();
        let runtime_path = get_runtime_path(); // runtime/runtime.c
        linker.link(&obj_path, &runtime_path, &output)?;

        // 5. 清理临时文件
        std::fs::remove_file(&obj_path)?;
    }
}
```

### 3.5 IR 层改造

需要支持对外部函数的调用声明：

```rust
// crates/xin-ir/src/ir.rs
pub enum Instruction {
    // ... 现有指令 ...

    // 调用外部函数
    CallExtern {
        result: Option<Value>,
        name: String,           // 函数名，如 "xin_print_int"
        args: Vec<Value>,
    },
}
```

### 3.6 Print/Println 处理

在 IR 生成阶段，将 `println(expr)` 转换为对应的运行时调用：

```rust
// crates/xin-ir/src/builder.rs
fn handle_println(&mut self, args: &[Expr]) {
    match args[0].ty {
        Type::Int => {
            // 生成 IR: CallExtern { name: "xin_print_int", args: [...] }
            // 生成 IR: CallExtern { name: "xin_println", args: [] }
        }
        Type::Float => {
            // xin_print_float + xin_println
        }
        Type::Bool => {
            // xin_print_bool + xin_println
        }
        Type::String => {
            // xin_print_str + xin_println
        }
        _ => panic!("Unsupported type for println"),
    }
}
```

## 4. 文件结构

```
xin/
├── src/
│   ├── main.rs           # CLI 入口（更新）
│   ├── compiler.rs       # 编译器编排（更新）
│   └── linker.rs         # 新增：链接器封装
├── crates/
│   ├── xin-codegen/
│   │   ├── src/
│   │   │   ├── lib.rs    # 导出（更新）
│   │   │   ├── cranelift.rs  # 现有 JIT 实现
│   │   │   └── aot.rs    # 新增：AOT 实现
│   └── xin-ir/
│       └── src/
│           ├── ir.rs     # IR 定义（更新）
│           └── builder.rs # IR 生成（更新）
└── runtime/
    └── runtime.c         # 新增：C 运行时
```

## 5. CLI 命令

```bash
# 编译为可执行文件
xin compile hello.xin -o hello

# 编译并直接运行
xin run hello.xin

# 查看 IR（调试）
xin compile hello.xin --emit-ir

# 只生成对象文件
xin compile hello.xin --emit-obj -o hello.o
```

## 6. 实现步骤

| 步骤 | 内容 | 依赖 |
|-----|------|------|
| 1 | 创建 runtime/runtime.c | 无 |
| 2 | 实现 src/linker.rs | 无 |
| 3 | 更新 IR：添加 CallExtern 指令 | 无 |
| 4 | 更新 IR Builder：处理 println/print | 步骤 3 |
| 5 | 实现 aot.rs：AOT CodeGenerator | 步骤 3 |
| 6 | 更新 compiler.rs：集成 AOT 流程 | 步骤 2, 4, 5 |
| 7 | 更新 main.rs：完善 CLI | 步骤 6 |
| 8 | 测试：编译运行 hello.xin | 步骤 7 |

## 7. 限制与后续工作

### 7.1 MVP 限制

- 仅支持基本类型（int, float, bool, string）的打印
- 不支持格式化字符串
- 不支持命令行参数读取
- 仅支持 macOS/Linux（Windows 需要额外处理）

### 7.2 后续工作

- 支持 `print` 函数（不换行）
- 支持 `format` 格式化字符串
- 支持 `readLine` 读取输入
- 支持命令行参数 `std.os.args()`
- 支持字符串字面量