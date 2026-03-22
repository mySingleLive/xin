//! Token definitions

use std::fmt;

/// Token kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // Literals
    IntLiteral,
    FloatLiteral,
    StringLiteral,
    TemplateString,
    BoolLiteral,

    // Identifiers and keywords
    Ident,
    Let,
    Var,
    Func,
    Struct,
    Interface,
    Implements,
    If,
    Else,
    For,
    In,
    Return,
    Null,
    Mut,
    Pub,
    Import,
    As,
    True,
    False,
    Move,

    // Types
    // Signed integers
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    // Unsigned integers
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    Byte,
    // Floats
    Float8,
    Float16,
    Float32,
    Float64,
    Float128,
    // Other types
    Bool,
    String,
    Char,
    Void,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqEq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    AndAnd,
    OrOr,
    Not,
    QuestionDot,
    QuestionQuestion,
    BangBang,
    ColonColon,
    Eq,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    Arrow,
    Colon,
    Semicolon,
    Comma,
    Dot,
    Question,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    // Special
    Eof,
    Unknown,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::IntLiteral => write!(f, "integer literal"),
            TokenKind::FloatLiteral => write!(f, "float literal"),
            TokenKind::StringLiteral => write!(f, "string literal"),
            TokenKind::TemplateString => write!(f, "template string"),
            TokenKind::BoolLiteral => write!(f, "boolean literal"),
            TokenKind::Ident => write!(f, "identifier"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Var => write!(f, "var"),
            TokenKind::Func => write!(f, "func"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Interface => write!(f, "interface"),
            TokenKind::Implements => write!(f, "implements"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::For => write!(f, "for"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Null => write!(f, "null"),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::Pub => write!(f, "pub"),
            TokenKind::Import => write!(f, "import"),
            TokenKind::As => write!(f, "as"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Move => write!(f, "move"),
            // Types - Signed integers
            TokenKind::Int8 => write!(f, "int8"),
            TokenKind::Int16 => write!(f, "int16"),
            TokenKind::Int32 => write!(f, "int32"),
            TokenKind::Int64 => write!(f, "int64"),
            TokenKind::Int128 => write!(f, "int128"),
            // Types - Unsigned integers
            TokenKind::UInt8 => write!(f, "uint8"),
            TokenKind::UInt16 => write!(f, "uint16"),
            TokenKind::UInt32 => write!(f, "uint32"),
            TokenKind::UInt64 => write!(f, "uint64"),
            TokenKind::UInt128 => write!(f, "uint128"),
            TokenKind::Byte => write!(f, "byte"),
            // Types - Floats
            TokenKind::Float8 => write!(f, "float8"),
            TokenKind::Float16 => write!(f, "float16"),
            TokenKind::Float32 => write!(f, "float32"),
            TokenKind::Float64 => write!(f, "float64"),
            TokenKind::Float128 => write!(f, "float128"),
            // Types - Other
            TokenKind::Bool => write!(f, "bool"),
            TokenKind::String => write!(f, "string"),
            TokenKind::Char => write!(f, "char"),
            TokenKind::Void => write!(f, "void"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::Ne => write!(f, "!="),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::Le => write!(f, "<="),
            TokenKind::Ge => write!(f, ">="),
            TokenKind::AndAnd => write!(f, "&&"),
            TokenKind::OrOr => write!(f, "||"),
            TokenKind::Not => write!(f, "!"),
            TokenKind::QuestionDot => write!(f, "?."),
            TokenKind::QuestionQuestion => write!(f, "??"),
            TokenKind::BangBang => write!(f, "!!"),
            TokenKind::ColonColon => write!(f, ":="),
            TokenKind::Eq => write!(f, "="),
            TokenKind::PlusEq => write!(f, "+="),
            TokenKind::MinusEq => write!(f, "-="),
            TokenKind::StarEq => write!(f, "*="),
            TokenKind::SlashEq => write!(f, "/="),
            TokenKind::PercentEq => write!(f, "%="),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Question => write!(f, "?"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::Eof => write!(f, "end of file"),
            TokenKind::Unknown => write!(f, "unknown"),
        }
    }
}

/// A token with location information
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, text: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            kind,
            text: text.into(),
            line,
            column,
        }
    }

    pub fn eof(line: usize, column: usize) -> Self {
        Self {
            kind: TokenKind::Eof,
            text: String::new(),
            line,
            column,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_type_tokens() {
        // 有符号整数
        assert_eq!(TokenKind::Int8.to_string(), "int8");
        assert_eq!(TokenKind::Int16.to_string(), "int16");
        assert_eq!(TokenKind::Int32.to_string(), "int32");
        assert_eq!(TokenKind::Int64.to_string(), "int64");
        assert_eq!(TokenKind::Int128.to_string(), "int128");

        // 无符号整数
        assert_eq!(TokenKind::UInt8.to_string(), "uint8");
        assert_eq!(TokenKind::UInt16.to_string(), "uint16");
        assert_eq!(TokenKind::UInt32.to_string(), "uint32");
        assert_eq!(TokenKind::UInt64.to_string(), "uint64");
        assert_eq!(TokenKind::UInt128.to_string(), "uint128");
        assert_eq!(TokenKind::Byte.to_string(), "byte");

        // 浮点数
        assert_eq!(TokenKind::Float8.to_string(), "float8");
        assert_eq!(TokenKind::Float16.to_string(), "float16");
        assert_eq!(TokenKind::Float32.to_string(), "float32");
        assert_eq!(TokenKind::Float64.to_string(), "float64");
        assert_eq!(TokenKind::Float128.to_string(), "float128");

        // 字符
        assert_eq!(TokenKind::Char.to_string(), "char");
    }
}