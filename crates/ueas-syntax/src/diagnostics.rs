//! Source span and structured syntax diagnostics for `ueas-syntax`.

use serde::{Deserialize, Serialize};

/// 1-indexed line/column and 0-indexed UTF-8 byte span in source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct SourceSpan {
    /// Start byte offset (inclusive).
    pub start_byte: usize,
    /// End byte offset (exclusive).
    pub end_byte: usize,
    /// 1-indexed start line.
    pub start_line: usize,
    /// 1-indexed start column.
    pub start_col: usize,
    /// 1-indexed end line.
    pub end_line: usize,
    /// 1-indexed end column.
    pub end_col: usize,
}

/// Line/column lookup index for fast byte-offset -> (line, col) resolution.
#[derive(Debug, Clone)]
pub struct LineIndex {
    line_starts: Vec<usize>,
    len: usize,
}

impl LineIndex {
    /// Build a line index from source text.
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (idx, b) in source.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(idx + 1);
            }
        }
        Self {
            line_starts,
            len: source.len(),
        }
    }

    /// Resolve a byte offset to 1-indexed `(line, col)`.
    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let offset = offset.min(self.len);
        let line_idx = self
            .line_starts
            .partition_point(|&start| start <= offset)
            .saturating_sub(1);
        let line_start = self.line_starts[line_idx];
        (line_idx + 1, offset.saturating_sub(line_start) + 1)
    }

    /// Build a `SourceSpan` from `[start_byte, end_byte)`.
    pub fn span(&self, start_byte: usize, end_byte: usize) -> SourceSpan {
        let (start_line, start_col) = self.line_col(start_byte);
        let (end_line, end_col) = self.line_col(end_byte);
        SourceSpan {
            start_byte,
            end_byte,
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }
}

/// A structured syntax error emitted by the parser.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxDiagnostic {
    /// Source span of the syntax error.
    pub span: SourceSpan,
    /// Human-readable error message.
    pub message: String,
    /// Optional fix suggestion.
    pub hint: Option<String>,
}
