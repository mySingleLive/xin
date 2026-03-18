//! Lexer error definitions

use thiserror::Error;
use xin_diagnostics::{Diagnostic, DiagnosticCode};

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("Unexpected character: '{0}'")]
    UnexpectedChar(char),

    #[error("Unterminated string")]
    UnterminatedString,

    #[error("Unterminated template string")]
    UnterminatedTemplate,

    #[error("Invalid number: {0}")]
    InvalidNumber(String),

    #[error("Invalid escape sequence: '{0}'")]
    InvalidEscape(char),
}

impl From<LexerError> for Diagnostic {
    fn from(err: LexerError) -> Self {
        let code = match &err {
            LexerError::UnexpectedChar(_) => DiagnosticCode::L001,
            LexerError::UnterminatedString => DiagnosticCode::L002,
            LexerError::InvalidNumber(_) => DiagnosticCode::L003,
            LexerError::UnterminatedTemplate => DiagnosticCode::L004,
            LexerError::InvalidEscape(_) => DiagnosticCode::L001,
        };
        Diagnostic::error(code, err.to_string())
    }
}