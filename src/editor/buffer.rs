use ropey::Rope;
use std::path::Path;

/// A text buffer backed by a rope data structure
#[derive(Debug, Clone)]
pub struct Buffer {
    /// The rope containing the text
    rope: Rope,
}

impl Buffer {
    /// Create a new empty buffer
    pub fn new() -> Self {
        Self { rope: Rope::new() }
    }

    /// Create a buffer from a string
    pub fn from_str(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
        }
    }

    /// Load buffer content from a file
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self::from_str(&text))
    }

    /// Save buffer content to a file
    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        let text = self.rope.to_string();
        std::fs::write(path, text)
    }

    /// Get the total number of lines
    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    /// Get the total number of characters
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.rope.len_chars() == 0
    }

    /// Get a line by index (0-based)
    pub fn line(&self, line_idx: usize) -> Option<ropey::RopeSlice> {
        if line_idx < self.rope.len_lines() {
            Some(self.rope.line(line_idx))
        } else {
            None
        }
    }

    /// Get the length of a line (in characters, excluding newline)
    pub fn line_len(&self, line_idx: usize) -> usize {
        if let Some(line) = self.line(line_idx) {
            let len = line.len_chars();
            // Subtract 1 if line ends with newline (not the last line)
            if len > 0 && line_idx < self.len_lines() - 1 {
                len.saturating_sub(1)
            } else {
                len
            }
        } else {
            0
        }
    }

    /// Convert a (line, column) position to a character index
    pub fn line_col_to_char(&self, line: usize, col: usize) -> usize {
        if line >= self.len_lines() {
            return self.len_chars();
        }

        let line_start = self.rope.line_to_char(line);
        let line_len = self.line_len(line);
        let col = col.min(line_len);

        line_start + col
    }

    /// Convert a character index to (line, column)
    pub fn char_to_line_col(&self, char_idx: usize) -> (usize, usize) {
        let char_idx = char_idx.min(self.len_chars());
        let line = self.rope.char_to_line(char_idx);
        let line_start = self.rope.line_to_char(line);
        let col = char_idx - line_start;
        (line, col)
    }

    /// Insert a character at the given character index
    pub fn insert_char(&mut self, char_idx: usize, ch: char) {
        let idx = char_idx.min(self.len_chars());
        self.rope.insert_char(idx, ch);
    }

    /// Insert a string at the given character index
    pub fn insert_str(&mut self, char_idx: usize, text: &str) {
        let idx = char_idx.min(self.len_chars());
        self.rope.insert(idx, text);
    }

    /// Delete a character at the given character index
    pub fn delete_char(&mut self, char_idx: usize) {
        if char_idx < self.len_chars() {
            self.rope.remove(char_idx..char_idx + 1);
        }
    }

    /// Delete a range of characters
    pub fn delete_range(&mut self, start: usize, end: usize) {
        let start = start.min(self.len_chars());
        let end = end.min(self.len_chars());
        if start < end {
            self.rope.remove(start..end);
        }
    }

    /// Get a slice of the buffer as a string
    pub fn slice_to_string(&self, start: usize, end: usize) -> String {
        let start = start.min(self.len_chars());
        let end = end.min(self.len_chars());
        if start < end {
            self.rope.slice(start..end).to_string()
        } else {
            String::new()
        }
    }

    /// Get the character at a given index
    pub fn char_at(&self, char_idx: usize) -> Option<char> {
        if char_idx < self.len_chars() {
            Some(self.rope.char(char_idx))
        } else {
            None
        }
    }

    /// Get the entire buffer as a string
    pub fn to_string(&self) -> String {
        self.rope.to_string()
    }

    /// Get a reference to the underlying rope
    pub fn rope(&self) -> &Rope {
        &self.rope
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_buffer() {
        let buf = Buffer::new();
        assert_eq!(buf.len_chars(), 0);
        assert_eq!(buf.len_lines(), 1); // Rope considers empty as 1 line
        assert!(buf.is_empty());
    }

    #[test]
    fn test_from_str() {
        let buf = Buffer::from_str("Hello\nWorld");
        assert_eq!(buf.len_lines(), 2);
        assert_eq!(buf.line(0).unwrap().to_string(), "Hello\n");
        assert_eq!(buf.line(1).unwrap().to_string(), "World");
    }

    #[test]
    fn test_insert_char() {
        let mut buf = Buffer::from_str("Hello");
        buf.insert_char(5, '!');
        assert_eq!(buf.to_string(), "Hello!");
    }

    #[test]
    fn test_delete_char() {
        let mut buf = Buffer::from_str("Hello!");
        buf.delete_char(5);
        assert_eq!(buf.to_string(), "Hello");
    }

    #[test]
    fn test_line_col_conversion() {
        let buf = Buffer::from_str("Hello\nWorld");
        assert_eq!(buf.line_col_to_char(0, 0), 0);
        assert_eq!(buf.line_col_to_char(0, 5), 5);
        assert_eq!(buf.line_col_to_char(1, 0), 6);
        assert_eq!(buf.line_col_to_char(1, 5), 11);

        assert_eq!(buf.char_to_line_col(0), (0, 0));
        assert_eq!(buf.char_to_line_col(5), (0, 5));
        assert_eq!(buf.char_to_line_col(6), (1, 0));
        assert_eq!(buf.char_to_line_col(11), (1, 5));
    }
}
