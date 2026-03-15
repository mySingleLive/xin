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

        // 3. 新增：写入临时对象文件
        let temp_dir = std::env::temp_dir();
        let obj_path = temp_dir.join("xin_output.o");
        std::fs::write(&obj_path, &obj_bytes)?;

        // 4. 新增：写入运行时到临时文件
        let runtime_path = crate::runtime::write_runtime_to_temp()?;

        // 5. 新增：链接
        let linker = Linker::new()?;
        linker.link(&obj_path, &runtime_path, &output)?;

        // 6. 清理临时文件
        let _ = std::fs::remove_file(&obj_path);
        let _ = std::fs::remove_file(&runtime_path);

        Ok(())
    }
}
```

### 3.5 IR 层改造

#### 3.5.1 IR Module 扩展

在 IR 模块级别添加外部函数声明和字符串常量表：

```rust
// crates/xin-ir/src/ir.rs
pub struct IRModule {
    pub functions: Vec<IRFunction>,
    pub extern_functions: Vec<ExternFunction>,  // 外部函数声明
    pub strings: Vec<String>,                    // 字符串常量表
}

pub struct ExternFunction {
    pub name: String,
    pub params: Vec<IRType>,
    pub return_type: Option<IRType>,
}
```

#### 3.5.2 Call 指令统一

使用现有的 `Call` 指令，通过 `is_extern` 字段区分内部和外部函数调用：

```rust
// crates/xin-ir/src/ir.rs
pub enum Instruction {
    // ... 现有指令 ...

    Call {
        result: Option<Value>,
        func: String,
        args: Vec<Value>,
        is_extern: bool,  // 新增：标识是否为外部函数
    },
}
```

#### 3.5.3 字符串字面量支持

MVP 阶段支持字符串字面量，通过数据段实现。新增 `StringConst` 指令：

```rust
// crates/xin-ir/src/ir.rs (Instruction 枚举新增)
pub enum Instruction {
    // ... 现有指令 ...

    StringConst {
        result: Value,
        string_index: usize,  // 索引到 IRModule.strings 表
    },
}
```

代码生成时，字符串放入只读数据段（`.rodata`），`StringConst` 返回字符串指针。

### 3.6 Print/Println 处理

在 IR 生成阶段，将 `println(expr)` 转换为对应的运行时调用：

```rust
// crates/xin-ir/src/builder.rs
fn handle_println(&mut self, args: &[Expr]) {
    match args[0].ty {
        Type::Int => {
            // 生成 IR: Call { func: "xin_print_int", args: [...], is_extern: true }
            // 生成 IR: Call { func: "xin_println", args: [], is_extern: true }
        }
        Type::Float => {
            // xin_print_float + xin_println
        }
        Type::Bool => {
            // xin_print_bool + xin_println
        }
        Type::String => {
            // 字符串参数通过 StringConst 获取指针
            // xin_print_str + xin_println
        }
        _ => panic!("Unsupported type for println"),
    }
}
```

### 3.7 运行时文件部署

运行时文件 `runtime.c` 的部署策略：

1. **开发阶段**：直接使用项目源码目录 `runtime/runtime.c`
2. **安装阶段**：编译时嵌入到编译器二进制中

```rust
// src/runtime.rs
use std::path::PathBuf;

pub fn get_runtime_source() -> &'static str {
    // 编译时通过 include_str! 嵌入
    include_str!("../runtime/runtime.c")
}

pub fn write_runtime_to_temp() -> Result<PathBuf, String> {
    let temp_dir = std::env::temp_dir();
    // 使用进程 ID 和时间戳避免并发冲突
    let pid = std::process::id();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Time error: {}", e))?
        .as_millis();
    let runtime_path = temp_dir.join(format!("xin_runtime_{}_{}.c", pid, timestamp));
    std::fs::write(&runtime_path, get_runtime_source())
        .map_err(|e| format!("Failed to write runtime: {}", e))?;
    Ok(runtime_path)
}
```

### 3.8 链接器错误处理

```rust
// src/linker.rs
impl Linker {
    pub fn new() -> Result<Self, String> {
        // 检测系统可用的 C 编译器
        let compilers = ["cc", "gcc", "clang"];
        for compiler in &compilers {
            if which::which(compiler).is_ok() {
                return Ok(Self { c_compiler: compiler.to_string() });
            }
        }
        Err("No C compiler found. Please install cc, gcc, or clang.".to_string())
    }

    pub fn link(&self, obj_path: &Path, runtime_path: &Path, output: &Path) -> Result<(), String> {
        let status = std::process::Command::new(&self.c_compiler)
            .arg(obj_path)
            .arg(runtime_path)
            .arg("-o").arg(output)
            .status()
            .map_err(|e| format!("Failed to run linker: {}", e))?;

        if !status.success() {
            return Err(format!("Linker failed with exit code: {:?}", status.code()));
        }
        Ok(())
    }
}
```

## 4. 文件结构

```
xin/
├── src/
│   ├── main.rs           # CLI 入口（更新）
│   ├── compiler.rs       # 编译器编排（更新）
│   ├── linker.rs         # 新增：链接器封装
│   └── runtime.rs        # 新增：运行时嵌入
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
| 2 | 创建 src/runtime.rs（嵌入运行时源码） | 步骤 1 |
| 3 | 实现 src/linker.rs | 无 |
| 4 | 更新 IR：添加外部函数声明、字符串常量表、Call 的 is_extern 字段 | 无 |
| 5 | 更新 IR Builder：处理 println/print，字符串字面量 | 步骤 4 |
| 6 | 实现 aot.rs：AOT CodeGenerator（含字符串数据段） | 步骤 4 |
| 7 | 更新 compiler.rs：集成 AOT 流程 | 步骤 2, 3, 5, 6 |
| 8 | 更新 main.rs：完善 CLI | 步骤 7 |
| 9 | 测试：编译运行 hello.xin | 步骤 8 |

## 7. 限制与后续工作

### 7.1 MVP 限制

- 仅支持基本类型（int, float, bool, string）的打印
- 字符串字面量仅支持 ASCII 字符（无转义序列处理）
- 不支持格式化字符串（`format` 函数）
- 不支持命令行参数读取
- 仅支持 macOS/Linux（Windows 需要额外处理链接器差异）

### 7.2 后续工作

- 支持 `print` 函数（不换行）
- 支持 `format` 格式化字符串
- 支持 `readLine` 读取输入
- 支持命令行参数 `std.os.args()`
- 支持字符串转义序列（`\n`, `\t` 等）
- 支持 UTF-8 字符串