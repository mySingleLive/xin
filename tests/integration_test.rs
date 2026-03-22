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