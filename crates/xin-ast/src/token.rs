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
    Int,
    Float,
    Bool,
    String,
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
            TokenKind::Int => write!(f, "int"),
            TokenKind::Float => write!(f, "float"),
            TokenKind::Bool => write!(f, "bool"),
            TokenKind::String => write!(f, "string"),
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