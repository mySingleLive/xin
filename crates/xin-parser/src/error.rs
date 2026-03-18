//! Parser error definitions

use thiserror::Error;
use xin_ast::TokenKind;
use xin_diagnostics::{Diagnostic, DiagnosticCode};

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Unexpected token: expected {expected}, found {found}")]
    UnexpectedToken {
        expected: String,
        found: TokenKind,
    },

    #[error("Expected expression")]
    ExpectedExpression,

    #[error("Expected statement")]
    ExpectedStatement,

    #[error("Missing closing delimiter: {0}")]
    MissingClosingDelimiter(char),

    #[error("Invalid assignment target")]
    InvalidAssignmentTarget,

    #[error("Lexer error: {0}")]
    LexerError(String),

    #[error("Invalid escape character: '{0}'")]
    InvalidEscape(char),

    #[error("Unclosed template expression")]
    UnclosedTemplateExpr,
}

impl ParserError {
    pub fn unexpected_token(expected: &str, found: TokenKind) -> Self {
        Self::UnexpectedToken {
            expected: expected.to_string(),
            found,
        }
    }
}

impl From<ParserError> for Diagnostic {
    fn from(err: ParserError) -> Self {
        let code = match &err {
            ParserError::UnexpectedToken { .. } => DiagnosticCode::P001,
            ParserError::ExpectedExpression => DiagnosticCode::P001,
            ParserError::ExpectedStatement => DiagnosticCode::P001,
            ParserError::MissingClosingDelimiter(_) => DiagnosticCode::P003,
            ParserError::InvalidAssignmentTarget => DiagnosticCode::P001,
            ParserError::LexerError(_) => DiagnosticCode::P001,
            ParserError::InvalidEscape(_) => DiagnosticCode::P001,
            ParserError::UnclosedTemplateExpr => DiagnosticCode::P003,
        };
        Diagnostic::error(code, err.to_string())
    }
}