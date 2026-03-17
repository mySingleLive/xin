//! Lexer implementation

use xin_ast::{Token, TokenKind};

use crate::LexerError;

/// Keywords mapping
const KEYWORDS: &[(&str, TokenKind)] = &[
    ("let", TokenKind::Let),
    ("var", TokenKind::Var),
    ("func", TokenKind::Func),
    ("struct", TokenKind::Struct),
    ("interface", TokenKind::Interface),
    ("implements", TokenKind::Implements),
    ("if", TokenKind::If),
    ("else", TokenKind::Else),
    ("for", TokenKind::For),
    ("in", TokenKind::In),
    ("return", TokenKind::Return),
    ("null", TokenKind::Null),
    ("mut", TokenKind::Mut),
    ("pub", TokenKind::Pub),
    ("import", TokenKind::Import),
    ("as", TokenKind::As),
    ("true", TokenKind::True),
    ("false", TokenKind::False),
    ("move", TokenKind::Move),
    ("int", TokenKind::Int),
    ("float", TokenKind::Float),
    ("bool", TokenKind::Bool),
    ("string", TokenKind::String),
    ("void", TokenKind::Void),
];

/// Lexical analyzer
pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    /// Tokenize the entire source
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace();
            if self.is_at_end() {
                break;
            }
            tokens.push(self.next_token()?);
        }

        tokens.push(Token::eof(self.line, self.column));
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexerError> {
        // Skip whitespace before processing
        self.skip_whitespace();
        if self.is_at_end() {
            return Ok(Token::eof(self.line, self.column));
        }

        let start_line = self.line;
        let start_col = self.column;
        let ch = self.advance();

        match ch {
            // Single-character tokens
            '(' => self.make_token(TokenKind::LParen, start_line, start_col),
            ')' => self.make_token(TokenKind::RParen, start_line, start_col),
            '{' => self.make_token(TokenKind::LBrace, start_line, start_col),
            '}' => self.make_token(TokenKind::RBrace, start_line, start_col),
            '[' => self.make_token(TokenKind::LBracket, start_line, start_col),
            ']' => self.make_token(TokenKind::RBracket, start_line, start_col),
            ',' => self.make_token(TokenKind::Comma, start_line, start_col),
            ';' => self.make_token(TokenKind::Semicolon, start_line, start_col),
            ':' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::ColonColon, start_line, start_col)
                } else {
                    self.make_token(TokenKind::Colon, start_line, start_col)
                }
            }
            '.' => self.make_token(TokenKind::Dot, start_line, start_col),
            '?' => {
                if self.match_char('.') {
                    self.make_token(TokenKind::QuestionDot, start_line, start_col)
                } else if self.match_char('?') {
                    self.make_token(TokenKind::QuestionQuestion, start_line, start_col)
                } else {
                    self.make_token(TokenKind::Question, start_line, start_col)
                }
            }

            // Operators
            '+' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::PlusEq, start_line, start_col)
                } else {
                    self.make_token(TokenKind::Plus, start_line, start_col)
                }
            }
            '-' => {
                if self.match_char('>') {
                    self.make_token(TokenKind::Arrow, start_line, start_col)
                } else if self.match_char('=') {
                    self.make_token(TokenKind::MinusEq, start_line, start_col)
                } else {
                    self.make_token(TokenKind::Minus, start_line, start_col)
                }
            }
            '*' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::StarEq, start_line, start_col)
                } else {
                    self.make_token(TokenKind::Star, start_line, start_col)
                }
            }
            '/' => {
                if self.match_char('/') {
                    // Line comment
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                    self.next_token()
                } else if self.match_char('*') {
                    // Block comment
                    self.skip_block_comment()?;
                    self.next_token()
                } else if self.match_char('=') {
                    self.make_token(TokenKind::SlashEq, start_line, start_col)
                } else {
                    self.make_token(TokenKind::Slash, start_line, start_col)
                }
            }
            '%' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::PercentEq, start_line, start_col)
                } else {
                    self.make_token(TokenKind::Percent, start_line, start_col)
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::EqEq, start_line, start_col)
                } else {
                    self.make_token(TokenKind::Eq, start_line, start_col)
                }
            }
            '!' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::Ne, start_line, start_col)
                } else if self.match_char('!') {
                    self.make_token(TokenKind::BangBang, start_line, start_col)
                } else {
                    self.make_token(TokenKind::Not, start_line, start_col)
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::Le, start_line, start_col)
                } else {
                    self.make_token(TokenKind::Lt, start_line, start_col)
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::Ge, start_line, start_col)
                } else {
                    self.make_token(TokenKind::Gt, start_line, start_col)
                }
            }
            '&' => {
                if self.match_char('&') {
                    self.make_token(TokenKind::AndAnd, start_line, start_col)
                } else {
                    Err(LexerError::UnexpectedChar('&'))
                }
            }
            '|' => {
                if self.match_char('|') {
                    self.make_token(TokenKind::OrOr, start_line, start_col)
                } else {
                    Err(LexerError::UnexpectedChar('|'))
                }
            }

            // String literal
            '"' => self.string_literal(start_line, start_col),

            // Number literals
            '0'..='9' => self.number_literal(start_line, start_col, ch),

            // Identifier or keyword
            'a'..='z' | 'A'..='Z' | '_' => self.identifier(start_line, start_col, ch),

            // Unknown
            _ => Err(LexerError::UnexpectedChar(ch)),
        }
    }

    fn string_literal(&mut self, start_line: usize, start_col: usize) -> Result<Token, LexerError> {
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != '"' {
            let ch = self.advance();
            if ch == '\\' {
                let escaped = self.advance();
                match escaped {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '"' => value.push('"'),
                    '\\' => value.push('\\'),
                    _ => return Err(LexerError::InvalidEscape(escaped)),
                }
            } else if ch == '\n' {
                self.line += 1;
                self.column = 1;
                value.push(ch);
            } else {
                value.push(ch);
            }
        }

        if self.is_at_end() {
            return Err(LexerError::UnterminatedString);
        }

        self.advance(); // closing "
        Ok(Token::new(TokenKind::StringLiteral, value, start_line, start_col))
    }

    fn number_literal(&mut self, start_line: usize, start_col: usize, first: char) -> Result<Token, LexerError> {
        let mut value = String::from(first);

        while !self.is_at_end() && self.peek().is_ascii_digit() {
            value.push(self.advance());
        }

        // Check for float
        if self.peek() == '.' && self.peek_next().map_or(false, |c| c.is_ascii_digit()) {
            value.push(self.advance()); // consume '.'
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                value.push(self.advance());
            }
            let _num: f64 = value.parse().map_err(|_| LexerError::InvalidNumber(value.clone()))?;
            return Ok(Token::new(TokenKind::FloatLiteral, value, start_line, start_col));
        }

        let _num: i64 = value.parse().map_err(|_| LexerError::InvalidNumber(value.clone()))?;
        Ok(Token::new(TokenKind::IntLiteral, value, start_line, start_col))
    }

    fn identifier(&mut self, start_line: usize, start_col: usize, first: char) -> Result<Token, LexerError> {
        let mut name = String::from(first);

        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            name.push(self.advance());
        }

        // Check for keyword
        let kind = KEYWORDS
            .iter()
            .find(|(kw, _)| *kw == name)
            .map(|(_, kind)| *kind)
            .unwrap_or(TokenKind::Ident);

        Ok(Token::new(kind, name, start_line, start_col))
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn skip_block_comment(&mut self) -> Result<(), LexerError> {
        let mut depth = 1;
        while !self.is_at_end() && depth > 0 {
            if self.peek() == '/' && self.peek_next() == Some('*') {
                self.advance();
                self.advance();
                depth += 1;
            } else if self.peek() == '*' && self.peek_next() == Some('/') {
                self.advance();
                self.advance();
                depth -= 1;
            } else {
                if self.peek() == '\n' {
                    self.line += 1;
                    self.column = 1;
                }
                self.advance();
            }
        }
        Ok(())
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn peek(&self) -> char {
        self.source.get(self.pos).copied().unwrap_or('\0')
    }

    fn peek_next(&self) -> Option<char> {
        self.source.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> char {
        let ch = self.source[self.pos];
        self.pos += 1;
        self.column += 1;
        ch
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.pos] != expected {
            false
        } else {
            self.pos += 1;
            self.column += 1;
            true
        }
    }

    fn make_token(&self, kind: TokenKind, line: usize, column: usize) -> Result<Token, LexerError> {
        Ok(Token::new(kind, kind.to_string(), line, column))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_integers() {
        let mut lexer = Lexer::new("42 0 123456");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 4); // 3 integers + EOF
        assert_eq!(tokens[0].kind, TokenKind::IntLiteral);
        assert_eq!(tokens[0].text, "42");
    }

    #[test]
    fn test_tokenize_keywords() {
        let mut lexer = Lexer::new("let var func if else for return");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(tokens[1].kind, TokenKind::Var);
        assert_eq!(tokens[2].kind, TokenKind::Func);
    }

    #[test]
    fn test_tokenize_operators() {
        let mut lexer = Lexer::new("+ - * / == != && || ->");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Plus);
        assert_eq!(tokens[4].kind, TokenKind::EqEq);
        assert_eq!(tokens[8].kind, TokenKind::Arrow);
    }

    #[test]
    fn test_tokenize_string() {
        let mut lexer = Lexer::new("\"hello world\"");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
        assert_eq!(tokens[0].text, "hello world");
    }

    #[test]
    fn test_tokenize_float() {
        let mut lexer = Lexer::new("3.14 0.5");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::FloatLiteral);
        assert_eq!(tokens[1].kind, TokenKind::FloatLiteral);
    }
}