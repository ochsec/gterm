/// Search functionality for the editor
use crate::editor::Document;

/// Represents a search match in the document
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchMatch {
    /// Line number (0-indexed)
    pub line: usize,
    /// Start column (0-indexed)
    pub start_col: usize,
    /// End column (exclusive, 0-indexed)
    pub end_col: usize,
}

/// Search state for the editor
#[derive(Debug, Clone)]
pub struct SearchState {
    /// Current search query
    pub query: String,
    /// Whether search is case-sensitive
    pub case_sensitive: bool,
    /// Whether to use regex
    pub use_regex: bool,
    /// Whether search bar is visible/active
    pub active: bool,
    /// All matches in the current document
    pub matches: Vec<SearchMatch>,
    /// Index of current match (for next/prev navigation)
    pub current_match: Option<usize>,
    /// Whether we're in replace mode
    pub replace_mode: bool,
    /// Replacement text
    pub replace_text: String,
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            case_sensitive: false,
            use_regex: false,
            active: false,
            matches: Vec::new(),
            current_match: None,
            replace_mode: false,
            replace_text: String::new(),
        }
    }

    /// Open the search bar
    pub fn open(&mut self) {
        self.active = true;
        self.replace_mode = false;
    }

    /// Open the search bar in replace mode
    pub fn open_replace(&mut self) {
        self.active = true;
        self.replace_mode = true;
    }

    /// Close the search bar
    pub fn close(&mut self) {
        self.active = false;
        self.matches.clear();
        self.current_match = None;
    }

    /// Update search query and find all matches
    pub fn search(&mut self, doc: &Document) {
        self.matches.clear();
        self.current_match = None;

        if self.query.is_empty() {
            return;
        }

        let query = if self.case_sensitive {
            self.query.clone()
        } else {
            self.query.to_lowercase()
        };

        // Search through all lines
        for line_idx in 0..doc.line_count() {
            let line_rope = match doc.buffer.line(line_idx) {
                Some(line) => line,
                None => continue,
            };
            let line_text: String = line_rope.chars().collect();
            let search_text = if self.case_sensitive {
                line_text.clone()
            } else {
                line_text.to_lowercase()
            };

            // Find all occurrences in this line
            let mut search_start = 0;
            while let Some(pos) = search_text[search_start..].find(&query) {
                let start_col = search_start + pos;
                let end_col = start_col + query.len();

                self.matches.push(SearchMatch {
                    line: line_idx,
                    start_col,
                    end_col,
                });

                search_start = start_col + 1;
                if search_start >= search_text.len() {
                    break;
                }
            }
        }

        // Set current match to first one if any exist
        if !self.matches.is_empty() {
            self.current_match = Some(0);
        }
    }

    /// Move to the next match
    pub fn next_match(&mut self) -> Option<SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }

        let next_idx = match self.current_match {
            Some(idx) => (idx + 1) % self.matches.len(),
            None => 0,
        };
        self.current_match = Some(next_idx);
        self.matches.get(next_idx).copied()
    }

    /// Move to the previous match
    pub fn prev_match(&mut self) -> Option<SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }

        let prev_idx = match self.current_match {
            Some(idx) => {
                if idx == 0 {
                    self.matches.len() - 1
                } else {
                    idx - 1
                }
            }
            None => self.matches.len() - 1,
        };
        self.current_match = Some(prev_idx);
        self.matches.get(prev_idx).copied()
    }

    /// Get the current match
    pub fn current(&self) -> Option<SearchMatch> {
        self.current_match
            .and_then(|idx| self.matches.get(idx).copied())
    }

    /// Find the next match after the given position
    pub fn find_next_from(&mut self, line: usize, col: usize) -> Option<SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }

        // Find the first match that comes after (line, col)
        for (idx, m) in self.matches.iter().enumerate() {
            if m.line > line || (m.line == line && m.start_col > col) {
                self.current_match = Some(idx);
                return Some(*m);
            }
        }

        // Wrap around to first match
        self.current_match = Some(0);
        self.matches.first().copied()
    }

    /// Find the previous match before the given position
    pub fn find_prev_from(&mut self, line: usize, col: usize) -> Option<SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }

        // Find the last match that comes before (line, col)
        for (idx, m) in self.matches.iter().enumerate().rev() {
            if m.line < line || (m.line == line && m.start_col < col) {
                self.current_match = Some(idx);
                return Some(*m);
            }
        }

        // Wrap around to last match
        let last_idx = self.matches.len() - 1;
        self.current_match = Some(last_idx);
        self.matches.last().copied()
    }

    /// Check if a position is within a match
    pub fn is_match(&self, line: usize, col: usize) -> bool {
        self.matches
            .iter()
            .any(|m| m.line == line && col >= m.start_col && col < m.end_col)
    }

    /// Check if a position is within the current match
    pub fn is_current_match(&self, line: usize, col: usize) -> bool {
        if let Some(m) = self.current() {
            m.line == line && col >= m.start_col && col < m.end_col
        } else {
            false
        }
    }

    /// Get match count info string
    pub fn match_info(&self) -> String {
        if self.matches.is_empty() {
            if self.query.is_empty() {
                String::new()
            } else {
                "No matches".to_string()
            }
        } else {
            let current = self.current_match.map(|i| i + 1).unwrap_or(0);
            format!("{}/{}", current, self.matches.len())
        }
    }

    /// Handle backspace in search input
    pub fn backspace(&mut self) {
        self.query.pop();
    }

    /// Handle character input in search input
    pub fn input_char(&mut self, c: char) {
        self.query.push(c);
    }

    /// Handle backspace in replace input
    pub fn replace_backspace(&mut self) {
        self.replace_text.pop();
    }

    /// Handle character input in replace input
    pub fn replace_input_char(&mut self, c: char) {
        self.replace_text.push(c);
    }
}
