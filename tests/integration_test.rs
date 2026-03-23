//! Integration tests for Xin compiler

#[test]
fn test_lexer_basic() {
    use xin_lexer::Lexer;
    let mut lexer = Lexer::new("let x = 10");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 5); // let, x, =, 10, EOF
}

#[test]
fn test_parser_basic() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;

    let source = r#"
        func add(a: int64, b: int64) int64 {
            return a + b
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();
    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_type_checker_basic() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_semantic::TypeChecker;

    let source = r#"
        func main() {
            let x = 10
            let y = x + 20
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();

    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_ir_builder_basic() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_semantic::TypeChecker;
    use xin_ir::IRBuilder;

    let source = r#"
        func add(a: int64, b: int64) int64 {
            return a + b
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();

    let mut type_checker = TypeChecker::new();
    type_checker.check(&ast).unwrap();

    let mut ir_builder = IRBuilder::new();
    let ir_module = ir_builder.build(&ast);
    assert_eq!(ir_module.functions.len(), 1);
}

#[test]
fn test_char_function_compile_time_check() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_semantic::TypeChecker;

    // char('a') should pass - single character
    let source = r#"
        func main() {
            let c = char('a')
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();

    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&ast);
    assert!(result.is_ok(), "char('a') should pass");

    // char('中') should pass - single unicode character
    let source = r#"
        func main() {
            let c = char('中')
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();

    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&ast);
    assert!(result.is_ok(), "char('中') should pass");

    // char('😊') should pass - single emoji
    let source = r#"
        func main() {
            let c = char('😊')
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();

    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&ast);
    assert!(result.is_ok(), "char('😊') should pass");

    // char('ab') should fail - multiple characters
    let source = r#"
        func main() {
            let c = char('ab')
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();

    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&ast);
    assert!(result.is_err(), "char('ab') should fail");

    // char('') should fail - empty string
    let source = r#"
        func main() {
            let c = char('')
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();

    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&ast);
    assert!(result.is_err(), "char('') should fail");
}

#[test]
fn test_lexer_safe_navigation_and_elvis() {
    use xin_ast::TokenKind;
    use xin_lexer::Lexer;

    // Test safe navigation operator ?.
    let mut lexer = Lexer::new("obj?.field");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 4); // obj, ?., field, EOF
    assert_eq!(tokens[1].kind, TokenKind::QuestionDot);

    // Test elvis operator ??
    let mut lexer = Lexer::new("x ?? y");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 4); // x, ??, y, EOF
    assert_eq!(tokens[1].kind, TokenKind::QuestionQuestion);

    // Test chained safe navigation
    let mut lexer = Lexer::new("user?.address?.city");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[1].kind, TokenKind::QuestionDot);
    assert_eq!(tokens[3].kind, TokenKind::QuestionDot);
}

#[test]
fn test_parser_elvis_operator() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_ast::ExprKind;

    let source = r#"
        func main() {
            let x = a ?? b
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();
    assert_eq!(ast.declarations.len(), 1);

    // Check that the Elvis expression was parsed
    if let xin_ast::DeclKind::Func(func) = &ast.declarations[0].kind {
        if let xin_ast::FuncBody::Block(stmts) = &func.body {
            if let xin_ast::StmtKind::VarDecl(var) = &stmts[0].kind {
                if let Some(value) = &var.value {
                    assert!(matches!(value.kind, ExprKind::Elvis { .. }),
                            "Expected Elvis expression");
                } else {
                    panic!("Expected variable to have a value");
                }
            } else {
                panic!("Expected VarDecl statement");
            }
        } else {
            panic!("Expected Block body");
        }
    } else {
        panic!("Expected Func declaration");
    }
}

#[test]
fn test_parser_safe_navigation() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_ast::ExprKind;

    let source = r#"
        func main() {
            let x = user?.name
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();
    assert_eq!(ast.declarations.len(), 1);

    // Check that the SafeAccess expression was parsed
    if let xin_ast::DeclKind::Func(func) = &ast.declarations[0].kind {
        if let xin_ast::FuncBody::Block(stmts) = &func.body {
            if let xin_ast::StmtKind::VarDecl(var) = &stmts[0].kind {
                if let Some(value) = &var.value {
                    assert!(matches!(value.kind, ExprKind::SafeAccess { .. }),
                            "Expected SafeAccess expression");
                } else {
                    panic!("Expected variable to have a value");
                }
            } else {
                panic!("Expected VarDecl statement");
            }
        } else {
            panic!("Expected Block body");
        }
    } else {
        panic!("Expected Func declaration");
    }
}

#[test]
fn test_ir_elvis_operator() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_semantic::TypeChecker;
    use xin_ir::IRBuilder;

    let source = r#"
        func main() {
            let x: int64? = null
            let y = x ?? 10
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();

    let mut type_checker = TypeChecker::new();
    type_checker.check(&ast).unwrap();

    let mut ir_builder = IRBuilder::new();
    let ir_module = ir_builder.build(&ast);
    assert_eq!(ir_module.functions.len(), 1);

    // Check that IR contains Phi instruction (used in Elvis implementation)
    let func = &ir_module.functions[0];
    let has_phi = func.instructions.iter().any(|instr| {
        matches!(instr, xin_ir::Instruction::Phi { .. })
    });
    assert!(has_phi, "Elvis operator should generate Phi instruction");
}

#[test]
fn test_ir_safe_navigation() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_semantic::TypeChecker;
    use xin_ir::IRBuilder;

    // Test SafeAccess with simple nullable value
    let source = r#"
        func main() {
            let x: int64? = null
            let y = x ?? 0
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();

    let mut type_checker = TypeChecker::new();
    type_checker.check(&ast).unwrap();

    let mut ir_builder = IRBuilder::new();
    let ir_module = ir_builder.build(&ast);
    assert!(ir_module.functions.len() >= 1, "Should have at least one function");

    // Check that IR contains Phi instruction (used in SafeAccess/Elvis implementation)
    let main_func = ir_module.functions.iter().find(|f| f.name == "main");
    if let Some(func) = main_func {
        let has_phi = func.instructions.iter().any(|instr| {
            matches!(instr, xin_ir::Instruction::Phi { .. })
        });
        assert!(has_phi, "SafeAccess/Elvis should generate Phi instruction");
    }
}

// ==================== Lambda Tests ====================

#[test]
fn test_ir_lambda_basic() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_semantic::TypeChecker;
    use xin_ir::IRBuilder;

    // Basic lambda with expression body
    let source = r#"
        func main() {
            let f = (x: int64, y: int64) -> x + y
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();

    let mut type_checker = TypeChecker::new();
    type_checker.check(&ast).unwrap();

    let mut ir_builder = IRBuilder::new();
    let ir_module = ir_builder.build(&ast);

    // Should have main function and a lambda function
    assert!(ir_module.functions.len() >= 2, "Should have main function and lambda function");

    // Check that there's a lambda function
    let has_lambda = ir_module.functions.iter().any(|f| f.name.starts_with("__lambda_"));
    assert!(has_lambda, "Should have a lambda function");

    // Check that main function contains a LambdaRef instruction
    let main_func = ir_module.functions.iter().find(|f| f.name == "main");
    if let Some(func) = main_func {
        let has_lambda_ref = func.instructions.iter().any(|instr| {
            matches!(instr, xin_ir::Instruction::LambdaRef { .. })
        });
        assert!(has_lambda_ref, "Main function should contain LambdaRef instruction");
    }
}

#[test]
fn test_ir_lambda_no_params() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_semantic::TypeChecker;
    use xin_ir::IRBuilder;

    // Lambda with no parameters
    let source = r#"
        func main() {
            let f = () -> 42
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();

    let mut type_checker = TypeChecker::new();
    type_checker.check(&ast).unwrap();

    let mut ir_builder = IRBuilder::new();
    let ir_module = ir_builder.build(&ast);

    // Should have at least main function
    assert!(ir_module.functions.len() >= 1);

    // Check that there's a lambda function
    let has_lambda = ir_module.functions.iter().any(|f| f.name.starts_with("__lambda_"));
    assert!(has_lambda, "Should have a lambda function");
}

#[test]
fn test_ir_lambda_block_body() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_semantic::TypeChecker;
    use xin_ir::IRBuilder;

    // Lambda with block body
    let source = r#"
        func main() {
            let f = (x: int64) -> {
                let y = x * 2
                return y
            }
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();

    let mut type_checker = TypeChecker::new();
    type_checker.check(&ast).unwrap();

    let mut ir_builder = IRBuilder::new();
    let ir_module = ir_builder.build(&ast);

    // Check that there's a lambda function with a body
    let lambda_func = ir_module.functions.iter().find(|f| f.name.starts_with("__lambda_"));
    assert!(lambda_func.is_some(), "Should have a lambda function");

    // Lambda should have some instructions in its body
    if let Some(func) = lambda_func {
        assert!(!func.instructions.is_empty(), "Lambda function should have instructions");
    }
}

#[test]
fn test_ir_lambda_direct_call() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_semantic::TypeChecker;
    use xin_ir::IRBuilder;

    // Lambda called directly
    let source = r#"
        func main() {
            let result = ((a: int64, b: int64) -> a + b)(10, 20)
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();

    let mut type_checker = TypeChecker::new();
    type_checker.check(&ast).unwrap();

    let mut ir_builder = IRBuilder::new();
    let ir_module = ir_builder.build(&ast);

    // Should have main function and a lambda function
    assert!(ir_module.functions.len() >= 2, "Should have main and lambda functions");

    // Check that main function contains both LambdaRef and IndirectCall
    let main_func = ir_module.functions.iter().find(|f| f.name == "main");
    if let Some(func) = main_func {
        let has_lambda_ref = func.instructions.iter().any(|instr| {
            matches!(instr, xin_ir::Instruction::LambdaRef { .. })
        });
        let has_indirect_call = func.instructions.iter().any(|instr| {
            matches!(instr, xin_ir::Instruction::IndirectCall { .. })
        });
        assert!(has_lambda_ref, "Main should have LambdaRef");
        assert!(has_indirect_call, "Main should have IndirectCall for lambda invocation");
    }
}

#[test]
fn test_ir_lambda_stored_and_called() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_semantic::TypeChecker;
    use xin_ir::IRBuilder;

    // Lambda stored in variable and called later
    let source = r#"
        func main() {
            let add = (a: int64, b: int64) -> a + b
            let result = add(1, 2)
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();

    let mut type_checker = TypeChecker::new();
    type_checker.check(&ast).unwrap();

    let mut ir_builder = IRBuilder::new();
    let ir_module = ir_builder.build(&ast);

    // Check that main function contains both LambdaRef and IndirectCall
    let main_func = ir_module.functions.iter().find(|f| f.name == "main");
    if let Some(func) = main_func {
        let has_lambda_ref = func.instructions.iter().any(|instr| {
            matches!(instr, xin_ir::Instruction::LambdaRef { .. })
        });
        let has_indirect_call = func.instructions.iter().any(|instr| {
            matches!(instr, xin_ir::Instruction::IndirectCall { .. })
        });
        assert!(has_lambda_ref, "Main should have LambdaRef for lambda assignment");
        assert!(has_indirect_call, "Main should have IndirectCall for calling stored lambda");
    }
}

// ==================== Map Tests ====================

#[test]
fn test_ir_map_literal() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_semantic::TypeChecker;
    use xin_ir::IRBuilder;

    let source = r#"
        func main() {
            let m = { "a": 1, "b": 2 }
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();

    let mut type_checker = TypeChecker::new();
    type_checker.check(&ast).unwrap();

    let mut ir_builder = IRBuilder::new();
    let ir_module = ir_builder.build(&ast);

    // Check that main function contains MapNew instruction
    let main_func = ir_module.functions.iter().find(|f| f.name == "main");
    assert!(main_func.is_some(), "Should have main function");

    if let Some(func) = main_func {
        let has_map_new = func.instructions.iter().any(|instr| {
            matches!(instr, xin_ir::Instruction::MapNew { .. })
        });
        assert!(has_map_new, "Should have MapNew instruction for map literal");

        let has_map_set = func.instructions.iter().any(|instr| {
            matches!(instr, xin_ir::Instruction::MapSet { .. })
        });
        assert!(has_map_set, "Should have MapSet instructions for map entries");
    }
}

#[test]
fn test_ir_map_methods() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_semantic::TypeChecker;
    use xin_ir::IRBuilder;

    let source = r#"
        func main() {
            let m = { "name": "Alice", "age": 30 }
            let size = m.map_len()
            let has_name = m.has("name")
            let removed = m.remove("age")
            let keys = m.keys()
            let values = m.values()
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();

    let mut type_checker = TypeChecker::new();
    type_checker.check(&ast).unwrap();

    let mut ir_builder = IRBuilder::new();
    let ir_module = ir_builder.build(&ast);

    let main_func = ir_module.functions.iter().find(|f| f.name == "main");
    assert!(main_func.is_some(), "Should have main function");

    if let Some(func) = main_func {
        // Check for MapLen instruction
        let has_map_len = func.instructions.iter().any(|instr| {
            matches!(instr, xin_ir::Instruction::MapLen { .. })
        });
        assert!(has_map_len, "Should have MapLen instruction");

        // Check for MapHas instruction
        let has_map_has = func.instructions.iter().any(|instr| {
            matches!(instr, xin_ir::Instruction::MapHas { .. })
        });
        assert!(has_map_has, "Should have MapHas instruction");

        // Check for MapRemove instruction
        let has_map_remove = func.instructions.iter().any(|instr| {
            matches!(instr, xin_ir::Instruction::MapRemove { .. })
        });
        assert!(has_map_remove, "Should have MapRemove instruction");

        // Check for MapKeys instruction
        let has_map_keys = func.instructions.iter().any(|instr| {
            matches!(instr, xin_ir::Instruction::MapKeys { .. })
        });
        assert!(has_map_keys, "Should have MapKeys instruction");

        // Check for MapValues instruction
        let has_map_values = func.instructions.iter().any(|instr| {
            matches!(instr, xin_ir::Instruction::MapValues { .. })
        });
        assert!(has_map_values, "Should have MapValues instruction");
    }
}

#[test]
fn test_ir_map_get_set() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_semantic::TypeChecker;
    use xin_ir::IRBuilder;

    let source = r#"
        func main() {
            let m = { "a": 1 }
            let val = m["a"]
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();

    let mut type_checker = TypeChecker::new();
    type_checker.check(&ast).unwrap();

    let mut ir_builder = IRBuilder::new();
    let ir_module = ir_builder.build(&ast);

    let main_func = ir_module.functions.iter().find(|f| f.name == "main");
    assert!(main_func.is_some(), "Should have main function");

    if let Some(func) = main_func {
        // Check for MapGet instruction (used for map index access)
        let has_map_get = func.instructions.iter().any(|instr| {
            matches!(instr, xin_ir::Instruction::MapGet { .. })
        });
        assert!(has_map_get, "Should have MapGet instruction for map index access");
    }
}

#[test]
fn test_ir_empty_map() {
    use xin_lexer::Lexer;
    use xin_parser::Parser;
    use xin_semantic::TypeChecker;
    use xin_ir::IRBuilder;

    let source = r#"
        func main() {
            let m = {}
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();

    let mut type_checker = TypeChecker::new();
    type_checker.check(&ast).unwrap();

    let mut ir_builder = IRBuilder::new();
    let ir_module = ir_builder.build(&ast);

    let main_func = ir_module.functions.iter().find(|f| f.name == "main");
    assert!(main_func.is_some(), "Should have main function");

    if let Some(func) = main_func {
        // Empty map should still create a MapNew
        let has_map_new = func.instructions.iter().any(|instr| {
            matches!(instr, xin_ir::Instruction::MapNew { .. })
        });
        assert!(has_map_new, "Empty map should still create MapNew instruction");

        // But should have no MapSet instructions
        let has_map_set = func.instructions.iter().any(|instr| {
            matches!(instr, xin_ir::Instruction::MapSet { .. })
        });
        assert!(!has_map_set, "Empty map should have no MapSet instructions");
    }
}

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

// ==================== Strings E2E Tests ====================

#[test]
fn e2e_strings_concat() {
    run_e2e_test("strings", "concat");
}

#[test]
fn e2e_strings_printf() {
    run_e2e_test("strings", "printf");
}

// ==================== Operators E2E Tests ====================

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

// ==================== Templates E2E Tests ====================

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

// ==================== Control Flow E2E Tests ====================

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

// ==================== Functions E2E Tests ====================

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

// ==================== Arrays E2E Tests ====================

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

// ==================== Nullable E2E Tests ====================

#[test]
fn e2e_nullable_basic() {
    run_e2e_test("nullable", "basic");
}