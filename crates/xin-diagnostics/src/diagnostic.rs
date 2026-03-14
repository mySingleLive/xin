//! Diagnostic definitions

use std::path::PathBuf;

/// Diagnostic severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
    Help,
}

/// Standardized diagnostic codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
    // Lexer errors (L001-L099)
    L001, // Unexpected character
    L002, // Unterminated string
    L003, // Invalid number

    // Parser errors (P001-P099)
    P001, // Unexpected token
    P002, // Expected token
    P003, // Missing closing delimiter

    // Semantic errors (S001-S099)
    S001, // Undefined variable
    S002, // Type mismatch
    S003, // Cannot assign to immutable
    S004, // Null safety violation

    // Ownership errors (O001-O099)
    O001, // Use after move
    O002, // Missing move keyword

    // Codegen errors (C001-C099)
    C001, // Failed to generate code
}

impl DiagnosticCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticCode::L001 => "E001",
            DiagnosticCode::L002 => "E002",
            DiagnosticCode::L003 => "E003",
            DiagnosticCode::P001 => "E101",
            DiagnosticCode::P002 => "E102",
            DiagnosticCode::P003 => "E103",
            DiagnosticCode::S001 => "E201",
            DiagnosticCode::S002 => "E202",
            DiagnosticCode::S003 => "E203",
            DiagnosticCode::S004 => "E204",
            DiagnosticCode::O001 => "E301",
            DiagnosticCode::O002 => "E302",
            DiagnosticCode::C001 => "E401",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            DiagnosticCode::L001 | DiagnosticCode::L002 | DiagnosticCode::L003 => "lexer",
            DiagnosticCode::P001 | DiagnosticCode::P002 | DiagnosticCode::P003 => "parser",
            DiagnosticCode::S001 | DiagnosticCode::S002 | DiagnosticCode::S003 | DiagnosticCode::S004 => "semantic",
            DiagnosticCode::O001 | DiagnosticCode::O002 => "ownership",
            DiagnosticCode::C001 => "codegen",
        }
    }
}

/// Source location
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self { line, column, offset }
    }

    pub fn start() -> Self {
        Self { line: 1, column: 1, offset: 0 }
    }
}

/// Source span
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

impl SourceSpan {
    pub fn new(start: SourceLocation, end: SourceLocation) -> Self {
        Self { start, end }
    }

    pub fn single(location: SourceLocation) -> Self {
        Self { start: location, end: location }
    }
}

/// A diagnostic message
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub code: DiagnosticCode,
    pub message: String,
    pub file: Option<PathBuf>,
    pub span: Option<SourceSpan>,
    pub hints: Vec<String>,
    pub related: Vec<Diagnostic>,
}

impl Diagnostic {
    pub fn error(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            code,
            message: message.into(),
            file: None,
            span: None,
            hints: Vec::new(),
            related: Vec::new(),
        }
    }

    pub fn warning(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            code,
            message: message.into(),
            file: None,
            span: None,
            hints: Vec::new(),
            related: Vec::new(),
        }
    }

    pub fn with_file(mut self, file: PathBuf) -> Self {
        self.file = Some(file);
        self
    }

    pub fn with_span(mut self, span: SourceSpan) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hints.push(hint.into());
        self
    }

    pub fn add_related(mut self, diagnostic: Diagnostic) -> Self {
        self.related.push(diagnostic);
        self
    }
}