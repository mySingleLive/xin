//! Diagnostic system for the Xin compiler
//!
//! Provides error reporting with source code snippets and suggestions.

mod diagnostic;
mod reporter;
mod snippet;

pub use diagnostic::{Diagnostic, DiagnosticLevel, DiagnosticCode, SourceLocation, SourceSpan};
pub use reporter::DiagnosticReporter;
pub use snippet::SourceSnippet;