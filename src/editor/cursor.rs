/// Represents the cursor position in a document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cursor {
    /// Line number (0-based)
    pub line: usize,
    /// Column number (0-based, in characters)
    pub col: usize,
    /// The "wanted" column - preserved when moving up/down through shorter lines
    pub wanted_col: usize,
}

impl Cursor {
    /// Create a new cursor at the origin
    pub fn new() -> Self {
        Self {
            line: 0,
            col: 0,
            wanted_col: 0,
        }
    }

    /// Create a cursor at a specific position
    pub fn at(line: usize, col: usize) -> Self {
        Self {
            line,
            col,
            wanted_col: col,
        }
    }

    /// Move the cursor to a new position
    pub fn move_to(&mut self, line: usize, col: usize) {
        self.line = line;
        self.col = col;
        self.wanted_col = col;
    }

    /// Move right by one character
    pub fn move_right(&mut self, line_len: usize, total_lines: usize) {
        if self.col < line_len {
            self.col += 1;
            self.wanted_col = self.col;
        } else if self.line + 1 < total_lines {
            // Move to beginning of next line
            self.line += 1;
            self.col = 0;
            self.wanted_col = 0;
        }
    }

    /// Move left by one character
    pub fn move_left(&mut self, prev_line_len: impl Fn(usize) -> usize) {
        if self.col > 0 {
            self.col -= 1;
            self.wanted_col = self.col;
        } else if self.line > 0 {
            // Move to end of previous line
            self.line -= 1;
            self.col = prev_line_len(self.line);
            self.wanted_col = self.col;
        }
    }

    /// Move up by one line
    pub fn move_up(&mut self, line_len: impl Fn(usize) -> usize) {
        if self.line > 0 {
            self.line -= 1;
            let len = line_len(self.line);
            self.col = self.wanted_col.min(len);
        }
    }

    /// Move down by one line
    pub fn move_down(&mut self, total_lines: usize, line_len: impl Fn(usize) -> usize) {
        if self.line + 1 < total_lines {
            self.line += 1;
            let len = line_len(self.line);
            self.col = self.wanted_col.min(len);
        }
    }

    /// Move to the beginning of the current line
    pub fn move_to_line_start(&mut self) {
        self.col = 0;
        self.wanted_col = 0;
    }

    /// Move to the end of the current line
    pub fn move_to_line_end(&mut self, line_len: usize) {
        self.col = line_len;
        self.wanted_col = self.col;
    }

    /// Move to the beginning of the document
    pub fn move_to_start(&mut self) {
        self.line = 0;
        self.col = 0;
        self.wanted_col = 0;
    }

    /// Move to the end of the document
    pub fn move_to_end(&mut self, total_lines: usize, last_line_len: usize) {
        self.line = total_lines.saturating_sub(1);
        self.col = last_line_len;
        self.wanted_col = self.col;
    }
}

/// Represents a text selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    /// Anchor position (where selection started)
    pub anchor: Cursor,
    /// Head position (where cursor currently is)
    pub head: Cursor,
}

impl Selection {
    /// Create a new selection with the same anchor and head (no selection)
    pub fn new(cursor: Cursor) -> Self {
        Self {
            anchor: cursor,
            head: cursor,
        }
    }

    /// Check if there is an actual selection (anchor != head)
    pub fn has_selection(&self) -> bool {
        self.anchor.line != self.head.line || self.anchor.col != self.head.col
    }

    /// Get the start and end of the selection (ordered)
    pub fn ordered(&self) -> (Cursor, Cursor) {
        if self.anchor.line < self.head.line
            || (self.anchor.line == self.head.line && self.anchor.col <= self.head.col)
        {
            (self.anchor, self.head)
        } else {
            (self.head, self.anchor)
        }
    }

    /// Collapse the selection to just the cursor position
    pub fn collapse(&mut self) {
        self.anchor = self.head;
    }

    /// Extend the selection to a new head position
    pub fn extend_to(&mut self, head: Cursor) {
        self.head = head;
    }

    /// Check if a position is within the selection
    pub fn contains(&self, line: usize, col: usize) -> bool {
        if !self.has_selection() {
            return false;
        }

        let (start, end) = self.ordered();

        if line < start.line || line > end.line {
            return false;
        }

        if line == start.line && line == end.line {
            return col >= start.col && col < end.col;
        }

        if line == start.line {
            return col >= start.col;
        }

        if line == end.line {
            return col < end.col;
        }

        true
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::new(Cursor::new())
    }
}
