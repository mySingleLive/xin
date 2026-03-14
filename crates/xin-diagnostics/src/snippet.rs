//! Source code snippet display

use std::ops::Range;

/// A snippet of source code for display
#[derive(Debug, Clone)]
pub struct SourceSnippet {
    pub source: String,
    pub line_start: usize,
    pub highlight_ranges: Vec<(Range<usize>, String)>,
}

impl SourceSnippet {
    pub fn new(source: String, line_start: usize) -> Self {
        Self {
            source,
            line_start,
            highlight_ranges: Vec::new(),
        }
    }

    pub fn add_highlight(mut self, range: Range<usize>, label: String) -> Self {
        self.highlight_ranges.push((range, label));
        self
    }

    /// Format the snippet for display
    pub fn format(&self) -> String {
        let lines: Vec<&str> = self.source.lines().collect();
        let mut result = String::new();

        for (idx, line) in lines.iter().enumerate() {
            let line_num = self.line_start + idx;
            result.push_str(&format!("{:4} | {}\n", line_num, line));

            // Add underlines for highlights
            if let Some((range, label)) = self.highlight_ranges.first() {
                if range.start >= idx && range.end <= idx + 1 {
                    let mut underline = String::new();
                    for _ in 0..line.len() {
                        underline.push('^');
                    }
                    result.push_str(&format!("     | {} {}\n", underline, label));
                }
            }
        }

        result
    }
}