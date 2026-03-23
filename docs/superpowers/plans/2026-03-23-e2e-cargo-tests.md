# E2E Cargo 测试实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 tests 目录下的 26 个 .xin 语法特性测试转换为 cargo 单元测试，实现端到端验证。

**Architecture:** 在 integration_test.rs 中添加一个通用的 E2E 测试辅助函数，然后为每个 .xin 测试文件创建独立的测试函数，调用编译器编译并执行代码，对比输出结果。

**Tech Stack:** Rust 标准库 (std::process::Command, std::fs, std::env)

---

## 文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `tests/integration_test.rs` | 修改 | 追加辅助函数和 26 个测试函数 |

---

### Task 1: 添加 E2E 测试辅助函数

**Files:**
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: 在文件末尾添加 E2E 测试辅助函数**

在 `tests/integration_test.rs` 文件末尾添加：

```rust
// ==================== E2E Tests ====================

/// Helper function to run an end-to-end test
fn run_e2e_test(category: &str, test_name: &str) {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    // Get project root directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let project_root = PathBuf::from(&manifest_dir);

    // Construct paths
    let test_dir = project_root.join("tests").join(category);
    let xin_file = test_dir.join(format!("{}.xin", test_name));
    let expected_file = test_dir.join(format!("{}.expected", test_name));

    // Verify files exist
    assert!(xin_file.exists(), "Test file not found: {:?}", xin_file);
    assert!(expected_file.exists(), "Expected file not found: {:?}", expected_file);

    // Create temp binary path
    let temp_binary = env::temp_dir().join(format!("xin_e2e_{}_{}", category, test_name));

    // Compile the xin file
    let compile_output = Command::new("cargo")
        .args(["run", "--", "compile", xin_file.to_str().unwrap(), "-o", temp_binary.to_str().unwrap()])
        .current_dir(&project_root)
        .output()
        .expect("Failed to execute cargo run");

    if !compile_output.status.success() {
        panic!(
            "Compilation failed for {}:{}\nstdout: {}\nstderr: {}",
            category,
            test_name,
            String::from_utf8_lossy(&compile_output.stdout),
            String::from_utf8_lossy(&compile_output.stderr)
        );
    }

    // Run the compiled binary
    let run_output = Command::new(&temp_binary)
        .output()
        .expect("Failed to execute compiled binary");

    if !run_output.status.success() {
        panic!(
            "Execution failed for {}:{}\nstdout: {}\nstderr: {}",
            category,
            test_name,
            String::from_utf8_lossy(&run_output.stdout),
            String::from_utf8_lossy(&run_output.stderr)
        );
    }

    // Read expected output
    let expected = fs::read_to_string(&expected_file)
        .expect("Failed to read expected file");

    // Get actual output
    let actual = String::from_utf8_lossy(&run_output.stdout);

    // Normalize outputs (strip trailing whitespace per line)
    let normalize = |s: &str| -> String {
        s.lines().map(|line| line.trim_end()).collect::<Vec<_>>().join("\n")
    };

    let expected_normalized = normalize(&expected);
    let actual_normalized = normalize(&actual);

    // Clean up temp binary
    let _ = fs::remove_file(&temp_binary);

    assert_eq!(
        expected_normalized,
        actual_normalized,
        "Output mismatch for {}:{}",
        category,
        test_name
    );
}
```

- [ ] **Step 2: 运行测试验证编译通过**

Run: `cargo test --test integration_test -- --nocapture 2>&1 | head -20`
Expected: 编译成功，现有测试仍然通过

- [ ] **Step 3: 提交**

```bash
git add tests/integration_test.rs
git commit -m "test: add e2e test helper function"
```

---

### Task 2: 添加 basic 目录测试 (4个)

**Files:**
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: 添加 basic 目录的 4 个测试函数**

在辅助函数后面添加：

```rust
#[test]
fn e2e_basic_arithmetic() {
    run_e2e_test("basic", "arithmetic");
}

#[test]
fn e2e_basic_type_conversion() {
    run_e2e_test("basic", "type_conversion");
}

#[test]
fn e2e_basic_types() {
    run_e2e_test("basic", "types");
}

#[test]
fn e2e_basic_variables() {
    run_e2e_test("basic", "variables");
}
```

- [ ] **Step 2: 运行测试验证**

Run: `cargo test --test integration_test e2e_basic -- --nocapture`
Expected: 4 个测试全部通过

- [ ] **Step 3: 提交**

```bash
git add tests/integration_test.rs
git commit -m "test: add basic category e2e tests"
```

---

### Task 3: 添加 strings 目录测试 (2个)

**Files:**
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: 添加 strings 目录的 2 个测试函数**

```rust
#[test]
fn e2e_strings_concat() {
    run_e2e_test("strings", "concat");
}

#[test]
fn e2e_strings_printf() {
    run_e2e_test("strings", "printf");
}
```

- [ ] **Step 2: 运行测试验证**

Run: `cargo test --test integration_test e2e_strings -- --nocapture`
Expected: 2 个测试全部通过

- [ ] **Step 3: 提交**

```bash
git add tests/integration_test.rs
git commit -m "test: add strings category e2e tests"
```

---

### Task 4: 添加 operators 目录测试 (3个)

**Files:**
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: 添加 operators 目录的 3 个测试函数**

```rust
#[test]
fn e2e_operators_comparison() {
    run_e2e_test("operators", "comparison");
}

#[test]
fn e2e_operators_logical() {
    run_e2e_test("operators", "logical");
}

#[test]
fn e2e_operators_unary() {
    run_e2e_test("operators", "unary");
}
```

- [ ] **Step 2: 运行测试验证**

Run: `cargo test --test integration_test e2e_operators -- --nocapture`
Expected: 3 个测试全部通过

- [ ] **Step 3: 提交**

```bash
git add tests/integration_test.rs
git commit -m "test: add operators category e2e tests"
```

---

### Task 5: 添加 templates 目录测试 (3个)

**Files:**
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: 添加 templates 目录的 3 个测试函数**

```rust
#[test]
fn e2e_templates_basic() {
    run_e2e_test("templates", "basic");
}

#[test]
fn e2e_templates_expressions() {
    run_e2e_test("templates", "expressions");
}

#[test]
fn e2e_templates_escape() {
    run_e2e_test("templates", "escape");
}
```

- [ ] **Step 2: 运行测试验证**

Run: `cargo test --test integration_test e2e_templates -- --nocapture`
Expected: 3 个测试全部通过

- [ ] **Step 3: 提交**

```bash
git add tests/integration_test.rs
git commit -m "test: add templates category e2e tests"
```

---

### Task 6: 添加 control_flow 目录测试 (4个)

**Files:**
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: 添加 control_flow 目录的 4 个测试函数**

```rust
#[test]
fn e2e_control_flow_break_continue() {
    run_e2e_test("control_flow", "break_continue");
}

#[test]
fn e2e_control_flow_for_loops() {
    run_e2e_test("control_flow", "for_loops");
}

#[test]
fn e2e_control_flow_if_else() {
    run_e2e_test("control_flow", "if_else");
}

#[test]
fn e2e_control_flow_if_else_branching() {
    run_e2e_test("control_flow", "if_else_branching");
}
```

- [ ] **Step 2: 运行测试验证**

Run: `cargo test --test integration_test e2e_control_flow -- --nocapture`
Expected: 4 个测试全部通过

- [ ] **Step 3: 提交**

```bash
git add tests/integration_test.rs
git commit -m "test: add control_flow category e2e tests"
```

---

### Task 7: 添加 functions 目录测试 (3个)

**Files:**
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: 添加 functions 目录的 3 个测试函数**

```rust
#[test]
fn e2e_functions_basic_funcs() {
    run_e2e_test("functions", "basic_funcs");
}

#[test]
fn e2e_functions_recursion() {
    run_e2e_test("functions", "recursion");
}

#[test]
fn e2e_functions_return_test() {
    run_e2e_test("functions", "return_test");
}
```

- [ ] **Step 2: 运行测试验证**

Run: `cargo test --test integration_test e2e_functions -- --nocapture`
Expected: 3 个测试全部通过

- [ ] **Step 3: 提交**

```bash
git add tests/integration_test.rs
git commit -m "test: add functions category e2e tests"
```

---

### Task 8: 添加 arrays 目录测试 (3个)

**Files:**
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: 添加 arrays 目录的 3 个测试函数**

```rust
#[test]
fn e2e_arrays_basic() {
    run_e2e_test("arrays", "basic");
}

#[test]
fn e2e_arrays_mutable() {
    run_e2e_test("arrays", "mutable");
}

#[test]
fn e2e_arrays_nested() {
    run_e2e_test("arrays", "nested");
}
```

- [ ] **Step 2: 运行测试验证**

Run: `cargo test --test integration_test e2e_arrays -- --nocapture`
Expected: 3 个测试全部通过

- [ ] **Step 3: 提交**

```bash
git add tests/integration_test.rs
git commit -m "test: add arrays category e2e tests"
```

---

### Task 9: 添加 nullable 目录测试 (1个)

**Files:**
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: 添加 nullable 目录的 1 个测试函数**

```rust
#[test]
fn e2e_nullable_basic() {
    run_e2e_test("nullable", "basic");
}
```

- [ ] **Step 2: 运行测试验证**

Run: `cargo test --test integration_test e2e_nullable -- --nocapture`
Expected: 1 个测试通过

- [ ] **Step 3: 提交**

```bash
git add tests/integration_test.rs
git commit -m "test: add nullable category e2e tests"
```

---

### Task 10: 添加 maps 目录测试 (2个)

**Files:**
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: 添加 maps 目录的 2 个测试函数**

```rust
#[test]
fn e2e_maps_basic() {
    run_e2e_test("maps", "basic");
}

#[test]
fn e2e_maps_methods() {
    run_e2e_test("maps", "methods");
}
```

- [ ] **Step 2: 运行测试验证**

Run: `cargo test --test integration_test e2e_maps -- --nocapture`
Expected: 2 个测试全部通过

- [ ] **Step 3: 提交**

```bash
git add tests/integration_test.rs
git commit -m "test: add maps category e2e tests"
```

---

### Task 11: 添加 floats 目录测试 (1个)

**Files:**
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: 添加 floats 目录的 1 个测试函数**

```rust
#[test]
fn e2e_floats_arithmetic() {
    run_e2e_test("floats", "arithmetic");
}
```

- [ ] **Step 2: 运行测试验证**

Run: `cargo test --test integration_test e2e_floats -- --nocapture`
Expected: 1 个测试通过

- [ ] **Step 3: 提交**

```bash
git add tests/integration_test.rs
git commit -m "test: add floats category e2e tests"
```

---

### Task 12: 运行全部 E2E 测试验证

- [ ] **Step 1: 运行全部 E2E 测试**

Run: `cargo test --test integration_test e2e -- --nocapture`
Expected: 全部 26 个 E2E 测试通过

- [ ] **Step 2: 运行完整测试套件**

Run: `cargo test --test integration_test`
Expected: 所有测试通过（包括现有测试和新增 E2E 测试）

---

## 测试清单

| 目录 | 测试函数 | 数量 |
|------|----------|------|
| basic | `e2e_basic_arithmetic`, `e2e_basic_type_conversion`, `e2e_basic_types`, `e2e_basic_variables` | 4 |
| strings | `e2e_strings_concat`, `e2e_strings_printf` | 2 |
| operators | `e2e_operators_comparison`, `e2e_operators_logical`, `e2e_operators_unary` | 3 |
| templates | `e2e_templates_basic`, `e2e_templates_expressions`, `e2e_templates_escape` | 3 |
| control_flow | `e2e_control_flow_break_continue`, `e2e_control_flow_for_loops`, `e2e_control_flow_if_else`, `e2e_control_flow_if_else_branching` | 4 |
| functions | `e2e_functions_basic_funcs`, `e2e_functions_recursion`, `e2e_functions_return_test` | 3 |
| arrays | `e2e_arrays_basic`, `e2e_arrays_mutable`, `e2e_arrays_nested` | 3 |
| nullable | `e2e_nullable_basic` | 1 |
| maps | `e2e_maps_basic`, `e2e_maps_methods` | 2 |
| floats | `e2e_floats_arithmetic` | 1 |
| **总计** | | **26** |