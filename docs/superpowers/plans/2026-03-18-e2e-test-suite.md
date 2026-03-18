# Xin E2E 测试套件实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 Xin 编译器创建端到端测试套件，覆盖所有已实现的语法特性。

**Architecture:** 基于 Shell 脚本的测试运行器，按功能分类组织测试用例，每个测试用例包含 `.xin` 源文件和 `.expected` 预期输出文件。测试脚本编译并运行每个测试，比对实际输出与预期输出。

**Tech Stack:** Shell (bash), xin 编译器

---

## Task 1: 创建测试目录结构

**Files:**
- Create: `tests/`
- Create: `tests/basic/`
- Create: `tests/strings/`
- Create: `tests/operators/`
- Create: `tests/control_flow/`
- Create: `tests/functions/`

- [ ] **Step 1: 创建测试目录**

Run: `mkdir -p tests/{basic,strings,operators,control_flow,functions}`

Expected: 目录创建成功，无错误输出

- [ ] **Step 2: 验证目录结构**

Run: `ls -la tests/`

Expected: 显示 basic, strings, operators, control_flow, functions 五个目录

- [ ] **Step 3: 提交**

```bash
git add tests/
git commit -m "chore: create e2e test directory structure"
```

---

## Task 2: 编写测试运行脚本

**Files:**
- Create: `tests/run_tests.sh`

- [ ] **Step 1: 创建测试脚本**

Create file `tests/run_tests.sh`:

```bash
#!/bin/bash

# Xin E2E Test Runner
# Usage: ./run_tests.sh [directory] [--all] [-v|--verbose] [-h|--help]

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
RESET='\033[0m'

# Configuration
XIN_COMPILER="cargo run --"
PHASE1_DIRS=("basic" "strings" "operators")
PHASE2_DIRS=("control_flow" "functions")
VERBOSE=false
RUN_ALL=false
TARGET_DIR=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --all)
            RUN_ALL=true
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [directory] [--all] [-v|--verbose] [-h|--help]"
            echo ""
            echo "Arguments:"
            echo "  directory    Run tests only in specified directory"
            echo "  --all        Run all tests including phase 2 directories"
            echo "  -v, --verbose  Show detailed output (compile commands, temp paths)"
            echo "  -h, --help   Show this help message"
            exit 0
            ;;
        *)
            TARGET_DIR=$1
            shift
            ;;
    esac
done

# Determine which directories to test
if [ -n "$TARGET_DIR" ]; then
    TEST_DIRS=("$TARGET_DIR")
elif [ "$RUN_ALL" = true ]; then
    TEST_DIRS=("${PHASE1_DIRS[@]}" "${PHASE2_DIRS[@]}")
else
    TEST_DIRS=("${PHASE1_DIRS[@]}")
fi

# Counters
PASSED=0
FAILED=0
FAILED_TEST=""

# Find xin executable
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Verbose output helper
verbose_log() {
    if [ "$VERBOSE" = true ]; then
        echo -e "${CYAN}[VERBOSE] $1${RESET}"
    fi
}

# Run a single test
run_test() {
    local test_file=$1
    local test_dir=$(dirname "$test_file")
    local test_name=$(basename "$test_file" .xin)
    local expected_file="$test_dir/$test_name.expected"

    # Get relative path for display
    local rel_path="${test_dir##*/}/$test_name"

    # Check if expected file exists
    if [ ! -f "$expected_file" ]; then
        echo -e "${RED}✗ FAILED${RESET}"
        echo "Missing expected file: $expected_file"
        return 1
    fi

    # Create temp directory for compilation
    local temp_dir=$(mktemp -d)
    local output_bin="$temp_dir/test_bin"

    verbose_log "Temp directory: $temp_dir"
    verbose_log "Output binary: $output_bin"

    # Compile
    cd "$PROJECT_ROOT"
    local compile_cmd="$XIN_COMPILER compile \"$test_file\" -o \"$output_bin\""
    verbose_log "Compile command: $compile_cmd"

    local compile_output
    if ! compile_output=$($XIN_COMPILER compile "$test_file" -o "$output_bin" 2>&1); then
        echo -e "${RED}✗ FAILED${RESET}"
        echo "Compilation failed:"
        echo "$compile_output"
        rm -rf "$temp_dir"
        return 1
    fi

    verbose_log "Compilation successful"

    # Run and capture output
    local actual_output
    if ! actual_output=$("$output_bin" 2>&1); then
        echo -e "${RED}✗ FAILED${RESET}"
        echo "Execution failed with exit code: $?"
        rm -rf "$temp_dir"
        return 1
    fi

    verbose_log "Execution successful"

    # Clean up
    rm -rf "$temp_dir"

    # Read expected output
    local expected_output
    expected_output=$(cat "$expected_file")

    # Compare outputs (trim trailing whitespace from each line)
    local actual_trimmed
    local expected_trimmed
    actual_trimmed=$(echo "$actual_output" | sed 's/[[:space:]]*$//')
    expected_trimmed=$(echo "$expected_output" | sed 's/[[:space:]]*$//')

    if [ "$actual_trimmed" = "$expected_trimmed" ]; then
        echo -e "${GREEN}✓${RESET}"
        return 0
    else
        echo -e "${RED}✗ FAILED${RESET}"
        echo ""
        echo "--- Expected ---"
        echo "$expected_trimmed"
        echo ""
        echo "--- Actual ---"
        echo "$actual_trimmed"
        return 1
    fi
}

# Main
echo "[Xin E2E Tests]"
echo ""

for dir in "${TEST_DIRS[@]}"; do
    if [ ! -d "$SCRIPT_DIR/$dir" ]; then
        continue
    fi

    for test_file in "$SCRIPT_DIR/$dir"/*.xin; do
        if [ ! -f "$test_file" ]; then
            continue
        fi

        test_name=$(basename "$test_file" .xin)
        rel_path="$dir/$test_name"

        printf "Running %s... " "$rel_path"

        if run_test "$test_file"; then
            ((PASSED++))
        else
            ((FAILED++))
            FAILED_TEST="$rel_path"
            echo ""
            echo "Test failed: $rel_path"
            echo "Stopped at first failure."
            echo ""
            echo "Summary: $PASSED passed, $FAILED failed"
            exit 1
        fi
    done
done

echo ""
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed! ($PASSED/$PASSED)${RESET}"
else
    echo "Summary: $PASSED passed, $FAILED failed"
    exit 1
fi
```

- [ ] **Step 2: 设置脚本执行权限**

Run: `chmod +x tests/run_tests.sh`

Expected: 无错误输出

- [ ] **Step 3: 测试脚本帮助信息**

Run: `cd tests && ./run_tests.sh --help`

Expected: 显示使用说明

- [ ] **Step 4: 提交**

```bash
git add tests/run_tests.sh
git commit -m "feat: add e2e test runner script"
```

---

## Task 3: 编写 basic/arithmetic.xin 测试用例

**Files:**
- Create: `tests/basic/arithmetic.xin`
- Create: `tests/basic/arithmetic.expected`

- [ ] **Step 1: 创建 arithmetic.xin**

Create file `tests/basic/arithmetic.xin`:

```xin
// Test arithmetic operations

func main() {
    // Integer arithmetic
    println(10 + 5)   // 15
    println(10 - 3)   // 7
    println(4 * 5)    // 20
    println(20 / 4)   // 5
    println(17 % 5)   // 2

    // Float arithmetic
    println(10.5 + 5.0)   // 15.5
    println(10.5 - 3.0)   // 7.5
    println(4.0 * 5.0)    // 20.0
    println(20.0 / 4.0)   // 5.0

    // Operator precedence
    println(2 + 3 * 4)    // 14
}
```

- [ ] **Step 2: 编译并运行获取实际输出**

Run: `cargo run -- compile tests/basic/arithmetic.xin -o /tmp/arith_test && /tmp/arith_test`

Expected: 显示实际输出（记录下来作为预期输出）

- [ ] **Step 3: 创建 arithmetic.expected（基于实际输出）**

Create file `tests/basic/arithmetic.expected`:

```
15
7
20
5
2
15.5
7.5
20
5
14
```

**重要说明**: Step 2 的实际输出可能与预设的 expected 内容略有差异。以 Step 2 的实际输出为准更新 .expected 文件内容。

- [ ] **Step 4: 验证测试通过**

Run: `cd tests && ./run_tests.sh basic`

Expected: `Running basic/arithmetic... ✓`

- [ ] **Step 5: 提交**

```bash
git add tests/basic/arithmetic.xin tests/basic/arithmetic.expected
git commit -m "test: add arithmetic operations test case"
```

---

## Task 4: 编写 basic/variables.xin 测试用例

**Files:**
- Create: `tests/basic/variables.xin`
- Create: `tests/basic/variables.expected`

- [ ] **Step 1: 创建 variables.xin**

Create file `tests/basic/variables.xin`:

```xin
// Test variable declarations

func main() {
    // Variable declarations with type inference
    let a = 42
    let b = 3.14
    let c = true
    let d = "hello"

    println(a)
    println(b)
    println(c)
    println(d)

    // Print using variables
    let x = 100
    println(x)
}
```

- [ ] **Step 2: 编译并运行获取实际输出**

Run: `cargo run -- compile tests/basic/variables.xin -o /tmp/var_test && /tmp/var_test`

Expected: 显示实际输出

- [ ] **Step 3: 创建 variables.expected（基于实际输出）**

Create file `tests/basic/variables.expected`:

```
42
3.14
true
hello
100
```

- [ ] **Step 4: 验证测试通过**

Run: `cd tests && ./run_tests.sh basic`

Expected: 所有 basic 测试通过

- [ ] **Step 5: 提交**

```bash
git add tests/basic/variables.xin tests/basic/variables.expected
git commit -m "test: add variable declarations test case"
```

---

## Task 5: 编写 basic/types.xin 测试用例

**Files:**
- Create: `tests/basic/types.xin`
- Create: `tests/basic/types.expected`

- [ ] **Step 1: 创建 types.xin**

Create file `tests/basic/types.xin`:

```xin
// Test basic types

func main() {
    // Integer literal
    println(42)

    // Float literal
    println(3.14)

    // Boolean literals
    println(true)
    println(false)

    // String literal
    println("hello world")

    // Print using variables of each type
    let i = 100
    let f = 2.5
    let b = false
    let s = "variable"

    println(i)
    println(f)
    println(b)
    println(s)
}
```

- [ ] **Step 2: 编译并运行获取实际输出**

Run: `cargo run -- compile tests/basic/types.xin -o /tmp/types_test && /tmp/types_test`

Expected: 显示实际输出

- [ ] **Step 3: 创建 types.expected（基于实际输出）**

Create file `tests/basic/types.expected`:

```
42
3.14
true
false
hello world
100
2.5
false
variable
```

- [ ] **Step 4: 验证测试通过**

Run: `cd tests && ./run_tests.sh basic`

Expected: 所有 basic 测试通过

- [ ] **Step 5: 提交**

```bash
git add tests/basic/types.xin tests/basic/types.expected
git commit -m "test: add basic types test case"
```

---

## Task 6: 编写 strings/concat.xin 测试用例

**Files:**
- Create: `tests/strings/concat.xin`
- Create: `tests/strings/concat.expected`

- [ ] **Step 1: 创建 concat.xin**

Create file `tests/strings/concat.xin`:

```xin
// Test string concatenation

func main() {
    // String + String
    println("Hello" + " World")

    // String + Int
    println("Count: " + 42)

    // Int + String
    println(100 + " points")

    // String + Float
    println("Pi = " + 3.14159)

    // String + Bool
    println("Flag: " + true)
}
```

- [ ] **Step 2: 编译并运行获取实际输出**

Run: `cargo run -- compile tests/strings/concat.xin -o /tmp/concat_test && /tmp/concat_test`

Expected: 显示实际输出

- [ ] **Step 3: 创建 concat.expected（基于实际输出）**

Create file `tests/strings/concat.expected`:

```
Hello World
Count: 42
100 points
Pi = 3.14159
Flag: true
```

- [ ] **Step 4: 验证测试通过**

Run: `cd tests && ./run_tests.sh strings`

Expected: `Running strings/concat... ✓`

- [ ] **Step 5: 提交**

```bash
git add tests/strings/concat.xin tests/strings/concat.expected
git commit -m "test: add string concatenation test case"
```

---

## Task 7: 编写 strings/printf.xin 测试用例

**Files:**
- Create: `tests/strings/printf.xin`
- Create: `tests/strings/printf.expected`

- [ ] **Step 1: 创建 printf.xin**

Create file `tests/strings/printf.xin`:

```xin
// Test printf format specifiers

func main() {
    printf("Integer: %d\n", 42)
    printf("Float: %f\n", 3.14)
    printf("String: %s\n", "test")
    printf("Bool: %b\n", true)
    printf("Hex: 0x%x\n", 255)
    printf("Octal: %o\n", 64)
}
```

- [ ] **Step 2: 编译并运行获取实际输出**

Run: `cargo run -- compile tests/strings/printf.xin -o /tmp/printf_test && /tmp/printf_test`

Expected: 显示实际输出

- [ ] **Step 3: 创建 printf.expected（基于实际输出）**

Create file `tests/strings/printf.expected`:

```
Integer: 42
Float: 3.14
String: test
Bool: true
Hex: 0xff
Octal: 100
```

- [ ] **Step 4: 验证测试通过**

Run: `cd tests && ./run_tests.sh strings`

Expected: 所有 strings 测试通过

- [ ] **Step 5: 提交**

```bash
git add tests/strings/printf.xin tests/strings/printf.expected
git commit -m "test: add printf format specifiers test case"
```

---

## Task 8: 编写 operators/comparison.xin 测试用例

**Files:**
- Create: `tests/operators/comparison.xin`
- Create: `tests/operators/comparison.expected`

- [ ] **Step 1: 创建 comparison.xin**

Create file `tests/operators/comparison.xin`:

```xin
// Test comparison operators

func main() {
    // Equality
    println(10 == 10)   // true
    println(10 == 5)    // false

    // Inequality
    println(10 != 5)    // true
    println(10 != 10)   // false

    // Less than
    println(5 < 10)     // true
    println(10 < 5)     // false

    // Greater than
    println(10 > 5)     // true
    println(5 > 10)     // false

    // Less than or equal
    println(10 <= 10)   // true
    println(5 <= 10)    // true
    println(15 <= 10)   // false

    // Greater than or equal
    println(10 >= 10)   // true
    println(15 >= 10)   // true
    println(5 >= 10)    // false
}
```

- [ ] **Step 2: 编译并运行获取实际输出**

Run: `cargo run -- compile tests/operators/comparison.xin -o /tmp/cmp_test && /tmp/cmp_test`

Expected: 显示实际输出

- [ ] **Step 3: 创建 comparison.expected（基于实际输出）**

Create file `tests/operators/comparison.expected`:

```
true
false
true
false
true
false
true
false
true
true
false
true
true
false
```

- [ ] **Step 4: 验证测试通过**

Run: `cd tests && ./run_tests.sh operators`

Expected: `Running operators/comparison... ✓`

- [ ] **Step 5: 提交**

```bash
git add tests/operators/comparison.xin tests/operators/comparison.expected
git commit -m "test: add comparison operators test case"
```

---

## Task 9: 编写 operators/logical.xin 测试用例

**Files:**
- Create: `tests/operators/logical.xin`
- Create: `tests/operators/logical.expected`

- [ ] **Step 1: 创建 logical.xin**

Create file `tests/operators/logical.xin`:

```xin
// Test logical operators

func main() {
    // AND
    println(true && true)    // true
    println(true && false)   // false
    println(false && true)   // false

    // OR
    println(false || true)   // true
    println(true || false)   // true
    println(true || true)    // true
}
```

- [ ] **Step 2: 编译并运行获取实际输出**

Run: `cargo run -- compile tests/operators/logical.xin -o /tmp/logic_test && /tmp/logic_test`

Expected: 显示实际输出

- [ ] **Step 3: 创建 logical.expected（基于实际输出）**

Create file `tests/operators/logical.expected`:

```
true
false
false
true
true
true
```

- [ ] **Step 4: 验证测试通过**

Run: `cd tests && ./run_tests.sh operators`

Expected: `Running operators/logical... ✓`

- [ ] **Step 5: 提交**

```bash
git add tests/operators/logical.xin tests/operators/logical.expected
git commit -m "test: add logical operators test case"
```

---

## Task 10: 编写 operators/unary.xin 测试用例

**Files:**
- Create: `tests/operators/unary.xin`
- Create: `tests/operators/unary.expected`

- [ ] **Step 1: 创建 unary.xin**

Create file `tests/operators/unary.xin`:

```xin
// Test unary operators

func main() {
    // Negation
    println(-42)      // -42
    println(-(-42))   // 42

    // Logical NOT
    println(!true)    // false
    println(!false)   // true
}
```

- [ ] **Step 2: 编译并运行获取实际输出**

Run: `cargo run -- compile tests/operators/unary.xin -o /tmp/unary_test && /tmp/unary_test`

Expected: 显示实际输出

- [ ] **Step 3: 创建 unary.expected（基于实际输出）**

Create file `tests/operators/unary.expected`:

```
-42
42
false
true
```

- [ ] **Step 4: 验证测试通过**

Run: `cd tests && ./run_tests.sh operators`

Expected: 所有 operators 测试通过

- [ ] **Step 5: 提交**

```bash
git add tests/operators/unary.xin tests/operators/unary.expected
git commit -m "test: add unary operators test case"
```

---

## Task 11: 运行完整的第一阶段测试并验证

**Files:**
- Modify: `tests/run_tests.sh` (如有需要)

- [ ] **Step 1: 运行所有第一阶段测试**

Run: `cd tests && ./run_tests.sh`

Expected: `All tests passed! (8/8)`

- [ ] **Step 2: 测试详细输出模式**

Run: `cd tests && ./run_tests.sh -v`

Expected: 显示详细测试过程（包含 [VERBOSE] 标记的调试信息）

- [ ] **Step 3: 测试指定目录**

Run: `cd tests && ./run_tests.sh basic`

Expected: 只运行 basic 目录测试，全部通过

- [ ] **Step 4: 测试 --all 参数（应跳过不存在的控制流测试）**

Run: `cd tests && ./run_tests.sh --all`

Expected: 运行所有测试，或提示控制流测试目录无测试文件

- [ ] **Step 5: 最终提交**

```bash
git add tests/
git commit -m "test: complete phase 1 e2e test suite"
```

---

## Task 12: 创建第二阶段测试用例模板（待编译器修复后启用）

**Files:**
- Create: `tests/control_flow/if_else.xin`
- Create: `tests/control_flow/if_else.expected`
- Create: `tests/control_flow/for_loops.xin`
- Create: `tests/control_flow/for_loops.expected`
- Create: `tests/functions/basic_funcs.xin`
- Create: `tests/functions/basic_funcs.expected`
- Create: `tests/functions/recursion.xin`
- Create: `tests/functions/recursion.expected`

- [ ] **Step 1: 创建 control_flow/if_else.xin 模板**

Create file `tests/control_flow/if_else.xin`:

```xin
// Test if/else statements
// NOTE: This test requires control flow code generation to be implemented

func main() {
    let a = 10

    if (a > 5) {
        println("greater")
    } else {
        println("less")
    }

    if (a < 5) {
        println("small")
    } else if (a == 10) {
        println("equal")
    } else {
        println("other")
    }

    // Nested if
    if (a > 0) {
        if (a == 10) {
            println("nested")
        }
    }

    // Conditional expression
    let result = a == 10 ? "ten" : "other"
    println(result)
}
```

- [ ] **Step 2: 创建 control_flow/if_else.expected**

Create file `tests/control_flow/if_else.expected`:

```
greater
equal
nested
ten
```

- [ ] **Step 3: 创建 control_flow/for_loops.xin 模板**

Create file `tests/control_flow/for_loops.xin`:

```xin
// Test for loops
// NOTE: This test requires control flow code generation to be implemented

func main() {
    // C-style for loop
    for (let i = 0; i < 5; i = i + 1) {
        println(i)
    }
    println("count: 5")

    // While-style for loop
    let j = 5
    for (j > 0) {
        println(j)
        j = j - 1
    }
    println("done")

    // Infinite loop with break (requires break implementation)
    // let k = 0
    // for {
    //     k = k + 1
    //     if (k > 3) {
    //         break
    //     }
    //     println(k)
    // }
    // println("infinite done")
}
```

- [ ] **Step 4: 创建 control_flow/for_loops.expected**

Create file `tests/control_flow/for_loops.expected`:

```
0
1
2
3
4
count: 5
5
4
3
2
1
done
```

- [ ] **Step 5: 创建 functions/basic_funcs.xin 模板**

Create file `tests/functions/basic_funcs.xin`:

```xin
// Test basic functions
// NOTE: This test requires function return value to be fixed

func sayHello() {
    println("hello")
}

func greet(name: string) {
    println("greeting: " + name)
}

func add(a: int, b: int) int {
    return a + b
}

func sum(a: int, b: int, c: int) int {
    return a + b + c
}

func nested() int {
    return add(20, 22)
}

func main() {
    sayHello()
    greet("Alice")
    println("result: " + add(10, 5))
    println("sum: " + sum(10, 10, 10))
    println("nested: " + nested())
}
```

- [ ] **Step 6: 创建 functions/basic_funcs.expected**

Create file `tests/functions/basic_funcs.expected`:

```
hello
greeting: Alice
result: 15
sum: 30
nested: 42
```

- [ ] **Step 7: 创建 functions/recursion.xin 模板**

Create file `tests/functions/recursion.xin`:

```xin
// Test recursive functions
// NOTE: This test requires control flow and function return value to be fixed

func fibonacci(n: int) int {
    if n <= 1 {
        return n
    }
    return fibonacci(n - 1) + fibonacci(n - 2)
}

func factorial(n: int) int {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}

func main() {
    println(fibonacci(10))
    println(factorial(5))
}
```

- [ ] **Step 8: 创建 functions/recursion.expected**

Create file `tests/functions/recursion.expected`:

```
55
120
```

- [ ] **Step 9: 提交第二阶段测试模板**

```bash
git add tests/control_flow/ tests/functions/
git commit -m "test: add phase 2 test case templates (pending compiler fixes)"
```

---

## Task 13: 更新 README 文档

**Files:**
- Modify: `README.md`

- [ ] **Step 1: 添加测试套件说明到 README**

在 README.md 中添加测试部分：

```markdown
## 测试

### 运行端到端测试

```bash
# 运行第一阶段测试（basic, strings, operators）
cd tests && ./run_tests.sh

# 运行所有测试（包括第二阶段）
cd tests && ./run_tests.sh --all

# 运行指定目录的测试
cd tests && ./run_tests.sh basic
```

### 测试目录结构

- `basic/` - 基础功能测试（算术运算、变量、类型）
- `strings/` - 字符串操作测试（拼接、格式化）
- `operators/` - 运算符测试（比较、逻辑、一元）
- `control_flow/` - 控制流测试（第二阶段，待编译器修复）
- `functions/` - 函数测试（第二阶段，待编译器修复）
```

- [ ] **Step 2: 提交**

```bash
git add README.md
git commit -m "docs: add e2e test suite documentation to README"
```

---

## Summary

**第一阶段（可立即执行）:**
- Task 1: 创建目录结构
- Task 2: 编写测试运行脚本
- Task 3-5: 编写 basic/ 测试用例
- Task 6-7: 编写 strings/ 测试用例
- Task 8-10: 编写 operators/ 测试用例
- Task 11: 运行完整测试验证
- Task 13: 更新文档

**第二阶段（待编译器修复后）:**
- Task 12: 启用控制流和函数测试

**预计测试用例数量**: 8 个（第一阶段）
**预计完成时间**: 第一阶段可立即完成