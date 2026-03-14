//! Diagnostic reporter

use std::io::{self, Write};
use std::path::PathBuf;

use crate::diagnostic::{Diagnostic, DiagnosticLevel};

/// Diagnostic reporter for formatting and displaying errors
pub struct DiagnosticReporter {
    source_cache: Vec<(PathBuf, String)>,
}

impl DiagnosticReporter {
    pub fn new() -> Self {
        Self {
            source_cache: Vec::new(),
        }
    }

    pub fn add_source(&mut self, path: PathBuf, source: String) {
        self.source_cache.push((path, source));
    }

    pub fn report(&self, diagnostic: &Diagnostic) -> String {
        let mut output = String::new();

        // Header
        let level_str = match diagnostic.level {
            DiagnosticLevel::Error => "error",
            DiagnosticLevel::Warning => "warning",
            DiagnosticLevel::Note => "note",
            DiagnosticLevel::Help => "help",
        };

        output.push_str(&format!(
            "{}[{}]: {}\n",
            level_str,
            diagnostic.code.as_str(),
            diagnostic.message
        ));

        // Location
        if let (Some(file), Some(span)) = (&diagnostic.file, &diagnostic.span) {
            output.push_str(&format!(
                "  --> {}:{}:{}\n",
                file.display(),
                span.start.line,
                span.start.column
            ));
        }

        // Hints
        for hint in &diagnostic.hints {
            output.push_str(&format!("  help: {}\n", hint));
        }

        output
    }

    pub fn print(&self, diagnostic: &Diagnostic) -> io::Result<()> {
        let report = self.report(diagnostic);
        let mut stderr = io::stderr();
        stderr.write_all(report.as_bytes())?;
        Ok(())
    }
}

impl Default for DiagnosticReporter {
    fn default() -> Self {
        Self::new()
    }
}