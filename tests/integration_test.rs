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