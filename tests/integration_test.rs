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
        func add(a: int, b: int) int {
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
        func add(a: int, b: int) int {
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