use serde::{Deserialize, Serialize};

/// Represents a source code location range with 1-indexed line/column
/// coordinates and 0-indexed byte offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceSpan {
    /// 1-indexed starting line number.
    pub start_line: usize,
    /// 1-indexed starting column number (in UTF-8 characters/bytes).
    pub start_col: usize,
    /// 1-indexed ending line number.
    pub end_line: usize,
    /// 1-indexed ending column number.
    pub end_col: usize,
    /// 0-indexed byte offset in the source file.
    pub byte_offset: usize,
    /// Length of the span in bytes.
    pub byte_len: usize,
}

impl SourceSpan {
    /// Creates a new complete source span.
    pub const fn new(
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
        byte_offset: usize,
        byte_len: usize,
    ) -> Self {
        Self {
            start_line,
            start_col,
            end_line,
            end_col,
            byte_offset,
            byte_len,
        }
    }

    /// Creates a span from line and column information without byte offsets.
    pub const fn from_lines(
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> Self {
        Self {
            start_line,
            start_col,
            end_line,
            end_col,
            byte_offset: 0,
            byte_len: 0,
        }
    }

    /// Creates a zero-length point span at a specific line and column.
    pub const fn point(line: usize, col: usize) -> Self {
        Self {
            start_line: line,
            start_col: col,
            end_line: line,
            end_col: col,
            byte_offset: 0,
            byte_len: 0,
        }
    }

    /// Returns true if this span covers zero length.
    pub const fn is_empty(&self) -> bool {
        self.byte_len == 0 && self.start_line == self.end_line && self.start_col == self.end_col
    }

    /// Checks if a 1-indexed (line, col) position falls inside this span.
    pub fn contains_position(&self, line: usize, col: usize) -> bool {
        if line < self.start_line || line > self.end_line {
            return false;
        }
        if line == self.start_line && col < self.start_col {
            return false;
        }
        if line == self.end_line && col > self.end_col {
            return false;
        }
        true
    }
}

impl Default for SourceSpan {
    fn default() -> Self {
        Self::point(1, 1)
    }
}

/// A lookup structure that maps byte offsets to 1-indexed line and column positions.
#[derive(Debug, Clone)]
pub struct LineIndex {
    /// Byte offsets where each line begins.
    line_starts: Vec<usize>,
    /// Total byte length of the indexed text.
    total_bytes: usize,
}

impl LineIndex {
    /// Builds a line index from the given source text.
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (offset, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(offset + 1);
            }
        }
        Self {
            line_starts,
            total_bytes: source.len(),
        }
    }

    /// Translates a 0-indexed byte offset to a (line, column) tuple (1-indexed).
    pub fn line_col(&self, byte_offset: usize) -> (usize, usize) {
        let offset = byte_offset.min(self.total_bytes);
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };

        let line_start_offset = self.line_starts[line_idx];
        let col = offset.saturating_sub(line_start_offset) + 1;
        (line_idx + 1, col)
    }

    /// Returns the start byte offset of a 1-indexed line.
    pub fn line_start_offset(&self, line: usize) -> Option<usize> {
        if line == 0 || line > self.line_starts.len() {
            None
        } else {
            Some(self.line_starts[line - 1])
        }
    }

    /// Constructs a `SourceSpan` from start and end byte offsets.
    pub fn span_from_offsets(&self, start_offset: usize, end_offset: usize) -> SourceSpan {
        let start = start_offset.min(self.total_bytes);
        let end = end_offset.max(start).min(self.total_bytes);
        let (start_line, start_col) = self.line_col(start);
        let (end_line, end_col) = self.line_col(end);

        SourceSpan::new(
            start_line,
            start_col,
            end_line,
            end_col,
            start,
            end.saturating_sub(start),
        )
    }

    /// Total number of lines.
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }
}
