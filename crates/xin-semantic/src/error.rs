//! Semantic error definitions

use thiserror::Error;
use xin_ast::Type;
use xin_diagnostics::{Diagnostic, DiagnosticCode, SourceSpan};

#[derive(Error, Debug)]
pub enum SemanticError {
    #[error("Undefined variable: '{0}'")]
    UndefinedVariable(String),

    #[error("Undefined type: '{0}'")]
    UndefinedType(String),

    #[error("Type mismatch: expected '{expected}', found '{found}'")]
    TypeMismatch { expected: Type, found: Type },

    #[error("Cannot assign to immutable variable: '{0}'")]
    CannotAssignImmutable(String),

    #[error("Null safety violation: '{0}' may be null")]
    NullSafetyViolation(String),

    #[error("Variable already defined: '{0}'")]
    VariableAlreadyDefined(String),

    #[error("Undefined function: '{0}'")]
    UndefinedFunction(String),

    #[error("Wrong number of arguments: expected {expected}, found {found}")]
    WrongNumberOfArguments { expected: usize, found: usize },

    #[error("Use after move: '{0}' has been moved")]
    UseAfterMove(String),

    #[error("Missing move keyword for ownership transfer")]
    MissingMoveKeyword,

    #[error("Invalid assignment target")]
    InvalidAssignmentTarget,

    #[error("unknown format specifier '%{0}'")]
    InvalidFormatSpecifier(char),

    #[error("printf argument count mismatch: expected {expected}, found {found}")]
    PrintfArgumentCountMismatch { expected: usize, found: usize },

    #[error("printf argument type mismatch: expected '{expected:?}', found '{found:?}'")]
    PrintfArgumentTypeMismatch { expected: Type, found: Type },

    #[error("cannot convert type '{ty}' to string")]
    CannotConvertToString { ty: Type, span: SourceSpan },

    #[error("type '{ty}' is not indexable")]
    NotIndexable { ty: Type, span: SourceSpan },

    #[error("cannot modify immutable array with method '{method}'")]
    ImmutableArrayModification { method: String, span: SourceSpan },

    #[error("cannot modify immutable map with method '{method}'")]
    ImmutableMapModification { method: String, span: SourceSpan },

    #[error("array element type mismatch at index {index}: expected '{expected}', found '{actual}'")]
    ArrayElementTypeMismatch {
        expected: Type,
        actual: Type,
        index: usize,
        span: SourceSpan,
    },

    #[error("char() argument must be a single character string, found: '{0}'")]
    InvalidCharLiteral(String),

    #[error("'break' statement not within a loop")]
    BreakOutsideLoop,

    #[error("'continue' statement not within a loop")]
    ContinueOutsideLoop,
}

impl From<SemanticError> for Diagnostic {
    fn from(err: SemanticError) -> Self {
        let code = match &err {
            SemanticError::UndefinedVariable(_) => DiagnosticCode::S001,
            SemanticError::UndefinedType(_) => DiagnosticCode::S001,
            SemanticError::TypeMismatch { .. } => DiagnosticCode::S002,
            SemanticError::CannotAssignImmutable(_) => DiagnosticCode::S003,
            SemanticError::NullSafetyViolation(_) => DiagnosticCode::S004,
            SemanticError::VariableAlreadyDefined(_) => DiagnosticCode::S001,
            SemanticError::UndefinedFunction(_) => DiagnosticCode::S001,
            SemanticError::WrongNumberOfArguments { .. } => DiagnosticCode::S002,
            SemanticError::UseAfterMove(_) => DiagnosticCode::O001,
            SemanticError::MissingMoveKeyword => DiagnosticCode::O002,
            SemanticError::InvalidAssignmentTarget => DiagnosticCode::S001,
            SemanticError::InvalidFormatSpecifier(_) => DiagnosticCode::S002,
            SemanticError::PrintfArgumentCountMismatch { .. } => DiagnosticCode::S002,
            SemanticError::PrintfArgumentTypeMismatch { .. } => DiagnosticCode::S002,
            SemanticError::CannotConvertToString { .. } => DiagnosticCode::S002,
            SemanticError::NotIndexable { .. } => DiagnosticCode::S002,
            SemanticError::ImmutableArrayModification { .. } => DiagnosticCode::S003,
            SemanticError::ImmutableMapModification { .. } => DiagnosticCode::S003,
            SemanticError::ArrayElementTypeMismatch { .. } => DiagnosticCode::S002,
            SemanticError::InvalidCharLiteral(_) => DiagnosticCode::S002,
            SemanticError::BreakOutsideLoop => DiagnosticCode::S002,
            SemanticError::ContinueOutsideLoop => DiagnosticCode::S002,
        };
        Diagnostic::error(code, err.to_string())
    }
}