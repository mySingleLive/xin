//! Test file for Task 8.1: Parser Map Syntax

use xin_lexer::Lexer;
use xin_parser::Parser;

#[test]
fn test_empty_map() {
    let source = r#"
        func main() {
            let m = {}
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();
    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_map_with_string_literal_keys() {
    let source = r#"
        func main() {
            let m = {"name": "Alice", "age": 30}
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();
    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_map_with_variable_keys() {
    let source = r#"
        func main() {
            let key = "name"
            let m = {key: "Alice"}
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();
    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_map_with_expression_keys() {
    let source = r#"
        func main() {
            let m = {getKey(): getValue()}
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();
    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_map_index_access() {
    let source = r#"
        func main() {
            let m = {"name": "Alice"}
            let x = m["name"]
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let ast = parser.parse().unwrap();
    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_map_dot_access_with_string() {
    let source = r#"
        func main() {
            let m = {"name": "Alice"}
            let x = m."name"
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let result = parser.parse();
    // This should work after implementing the feature
    assert!(result.is_ok(), "Should parse m.\"name\" successfully: {:?}", result);
}

#[test]
fn test_map_dot_access_with_char() {
    let source = r#"
        func main() {
            let m = {"name": "Alice"}
            let x = m.'name'
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let result = parser.parse();
    // This should work after implementing the feature
    assert!(result.is_ok(), "Should parse m.'name' successfully: {:?}", result);
}

#[test]
fn test_map_dot_access_with_template() {
    let source = r#"
        func main() {
            let prefix = "user"
            let m = {"user_name": "Alice"}
            let x = m.`{prefix}_name`
        }
    "#;
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer).unwrap();
    let result = parser.parse();
    // This should work after implementing the feature
    assert!(result.is_ok(), "Should parse m with template string key successfully: {:?}", result);
}