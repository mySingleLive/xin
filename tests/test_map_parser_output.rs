//! Test file to verify Parser output for Map syntax

use xin_lexer::Lexer;
use xin_parser::Parser;
use xin_ast::DeclKind;

fn parse_and_verify(source: &str) {
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();

    // Just verify that parsing succeeds
    println!("Successfully parsed {} declarations", ast.declarations.len());

    // Print AST for debugging
    for decl in &ast.declarations {
        match &decl.kind {
            DeclKind::Func(func) => {
                println!("Function: {} with {} params", func.name, func.params.len());
            }
            _ => {}
        }
    }
}

#[test]
fn test_parser_map_empty() {
    let source = r#"
        func main() {
            let m = {}
        }
    "#;
    parse_and_verify(source);
}

#[test]
fn test_parser_map_string_keys() {
    let source = r#"
        func main() {
            let m = {"name": "Alice", "age": 30}
        }
    "#;
    parse_and_verify(source);
}

#[test]
fn test_parser_map_variable_keys() {
    let source = r#"
        func main() {
            let key = "name"
            let m = {key: "Alice"}
        }
    "#;
    parse_and_verify(source);
}

#[test]
fn test_parser_map_expression_keys() {
    let source = r#"
        func main() {
            let m = {getKey(): "value"}
        }

        func getKey() string {
            return "key"
        }
    "#;
    parse_and_verify(source);
}

#[test]
fn test_parser_map_access() {
    let source = r#"
        func main() {
            let m = {"name": "Alice"}
            let a = m["name"]
            let b = m."name"
            let c = m.'name'
            let d = m.`name`
        }
    "#;
    parse_and_verify(source);
}