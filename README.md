# Xin 编程语言

Xin 是一种静态编译、静态类型的系统编程语言，结合 Rust 的内存安全保证和 Go 的语法简洁性。

## 特性

- **内存安全，心智轻松** - 编译期检查 + 智能指针，无需手动管理内存
- **语法友好，学习曲线平缓** - 减少 Rust 那种复杂的生命周期标注
- **空安全默认** - 变量默认不可空，可空类型显式标记
- **不可变优先** - 变量和对象默认不可变

## 快速开始

```bash
# 编译项目
cargo build

# 运行测试
cargo test

# 运行端到端测试套件
./tests/run_tests.sh

# 编译 Xin 源文件
cargo run -- compile examples/fibonacci.xin -o fibonacci
```

## 示例

```xin
struct User {
    name: string
    age: int

    func greet() string {
        return "Hello, " + self.name
    }
}

func main() {
    let u = User { name: "Alice", age: 30 }
}
```

## 编译器架构

```
源码 → Lexer → Parser → AST → Semantic Analysis → IR Generation → Cranelift → 机器码
```

## 状态

目前处于 MVP 阶段，实现了：
- 词法分析
- 语法分析
- 类型检查
- IR 生成
- Cranelift 代码生成

## 端到端测试

测试套件位于 `tests/` 目录，包含 `.xin` 源文件和对应的 `.expected` 预期输出文件。

### 运行测试

```bash
# 运行第一阶段测试（基础功能）
./tests/run_tests.sh

# 运行指定目录测试
./tests/run_tests.sh basic
./tests/run_tests.sh strings
./tests/run_tests.sh operators

# 显示详细输出
./tests/run_tests.sh -v

# 运行所有测试（包括第二阶段）
./tests/run_tests.sh --all
```

### 测试目录

| 目录 | 状态 | 说明 |
|------|------|------|
| `basic/` | ✅ 启用 | 基础功能：算术运算、变量、类型 |
| `strings/` | ✅ 启用 | 字符串操作：拼接、printf |
| `operators/` | ✅ 启用 | 运算符：比较、一元运算 |
| `control_flow/` | ⏳ 待启用 | 控制流：if/else、循环 |
| `functions/` | ⏳ 待启用 | 函数：参数、返回值、递归 |

### 添加新测试

1. 在对应目录创建 `.xin` 文件
2. 编译并运行获取实际输出
3. 创建同名的 `.expected` 文件
4. 运行测试验证通过

## 许可证

MIT