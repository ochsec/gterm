use super::{Buffer, Cursor, Selection};
use std::path::PathBuf;

/// Line ending style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineEnding {
    #[default]
    Lf, // Unix: \n
    CrLf, // Windows: \r\n
    Cr,   // Old Mac: \r
}

impl LineEnding {
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::CrLf => "\r\n",
            LineEnding::Cr => "\r",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            LineEnding::Lf => "LF",
            LineEnding::CrLf => "CRLF",
            LineEnding::Cr => "CR",
        }
    }

    /// Detect line ending from text
    pub fn detect(text: &str) -> Self {
        if text.contains("\r\n") {
            LineEnding::CrLf
        } else if text.contains('\r') {
            LineEnding::Cr
        } else {
            LineEnding::Lf
        }
    }
}

/// A document represents an open file with its buffer, cursor, and metadata
#[derive(Debug, Clone)]
pub struct Document {
    /// The text buffer
    pub buffer: Buffer,
    /// Current cursor position
    pub cursor: Cursor,
    /// Current selection (may be collapsed to cursor)
    pub selection: Selection,
    /// File path (None for new untitled documents)
    pub path: Option<PathBuf>,
    /// Whether the document has unsaved changes
    pub modified: bool,
    /// Line ending style
    pub line_ending: LineEnding,
    /// Character encoding (we only support UTF-8 for now)
    pub encoding: String,
    /// Detected or set filetype
    pub filetype: String,
    /// Vertical scroll offset (first visible line)
    pub scroll_y: usize,
    /// Horizontal scroll offset (first visible column)
    pub scroll_x: usize,
    /// Insert mode (true) or overwrite mode (false)
    pub insert_mode: bool,
}

impl Document {
    /// Create a new empty document
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            cursor: Cursor::new(),
            selection: Selection::default(),
            path: None,
            modified: false,
            line_ending: LineEnding::default(),
            encoding: "UTF-8".to_string(),
            filetype: "Plain Text".to_string(),
            scroll_y: 0,
            scroll_x: 0,
            insert_mode: true,
        }
    }

    /// Create a document from a string (for new unsaved documents)
    pub fn from_str(text: &str) -> Self {
        let line_ending = LineEnding::detect(text);
        Self {
            buffer: Buffer::from_str(text),
            cursor: Cursor::new(),
            selection: Selection::default(),
            path: None,
            modified: true,
            line_ending,
            encoding: "UTF-8".to_string(),
            filetype: "Plain Text".to_string(),
            scroll_y: 0,
            scroll_x: 0,
            insert_mode: true,
        }
    }

    /// Open a document from a file
    pub fn open(path: PathBuf) -> std::io::Result<Self> {
        let text = std::fs::read_to_string(&path)?;
        let line_ending = LineEnding::detect(&text);
        let filetype = detect_filetype(&path);

        Ok(Self {
            buffer: Buffer::from_str(&text),
            cursor: Cursor::new(),
            selection: Selection::default(),
            path: Some(path),
            modified: false,
            line_ending,
            encoding: "UTF-8".to_string(),
            filetype,
            scroll_y: 0,
            scroll_x: 0,
            insert_mode: true,
        })
    }

    /// Save the document to its file path
    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(path) = &self.path {
            self.buffer.save_to_file(path)?;
            self.modified = false;
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No file path set",
            ))
        }
    }

    /// Save the document to a new path
    pub fn save_as(&mut self, path: PathBuf) -> std::io::Result<()> {
        self.buffer.save_to_file(&path)?;
        self.filetype = detect_filetype(&path);
        self.path = Some(path);
        self.modified = false;
        Ok(())
    }

    /// Get the document title (filename or "untitled")
    pub fn title(&self) -> String {
        if let Some(path) = &self.path {
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("untitled")
                .to_string()
        } else {
            "untitled".to_string()
        }
    }

    /// Get the number of lines
    pub fn line_count(&self) -> usize {
        self.buffer.len_lines()
    }

    /// Get the length of a specific line
    pub fn line_len(&self, line: usize) -> usize {
        self.buffer.line_len(line)
    }

    /// Insert a character at the cursor position
    pub fn insert_char(&mut self, ch: char) {
        // Delete selection first if any
        if self.selection.has_selection() {
            self.delete_selection();
        }

        let char_idx = self
            .buffer
            .line_col_to_char(self.cursor.line, self.cursor.col);

        if self.insert_mode {
            self.buffer.insert_char(char_idx, ch);
        } else {
            // Overwrite mode
            if char_idx < self.buffer.len_chars() {
                let current = self.buffer.char_at(char_idx);
                if current != Some('\n') {
                    self.buffer.delete_char(char_idx);
                }
            }
            self.buffer.insert_char(char_idx, ch);
        }

        // Move cursor
        if ch == '\n' {
            self.cursor.line += 1;
            self.cursor.col = 0;
            self.cursor.wanted_col = 0;
        } else {
            self.cursor.col += 1;
            self.cursor.wanted_col = self.cursor.col;
        }

        self.selection = Selection::new(self.cursor);
        self.modified = true;
    }

    /// Insert a string at the cursor position
    pub fn insert_str(&mut self, text: &str) {
        if self.selection.has_selection() {
            self.delete_selection();
        }

        let char_idx = self
            .buffer
            .line_col_to_char(self.cursor.line, self.cursor.col);
        self.buffer.insert_str(char_idx, text);

        // Update cursor position based on inserted text
        let new_idx = char_idx + text.chars().count();
        let (line, col) = self.buffer.char_to_line_col(new_idx);
        self.cursor.line = line;
        self.cursor.col = col;
        self.cursor.wanted_col = col;

        self.selection = Selection::new(self.cursor);
        self.modified = true;
    }

    /// Delete the character before the cursor (backspace)
    pub fn backspace(&mut self) {
        if self.selection.has_selection() {
            self.delete_selection();
            return;
        }

        if self.cursor.line == 0 && self.cursor.col == 0 {
            return; // Nothing to delete
        }

        let char_idx = self
            .buffer
            .line_col_to_char(self.cursor.line, self.cursor.col);
        if char_idx > 0 {
            self.buffer.delete_char(char_idx - 1);
            let (line, col) = self.buffer.char_to_line_col(char_idx - 1);
            self.cursor.line = line;
            self.cursor.col = col;
            self.cursor.wanted_col = col;
            self.selection = Selection::new(self.cursor);
            self.modified = true;
        }
    }

    /// Delete the character at the cursor (delete key)
    pub fn delete(&mut self) {
        if self.selection.has_selection() {
            self.delete_selection();
            return;
        }

        let char_idx = self
            .buffer
            .line_col_to_char(self.cursor.line, self.cursor.col);
        if char_idx < self.buffer.len_chars() {
            self.buffer.delete_char(char_idx);
            self.modified = true;
        }
    }

    /// Delete the current selection
    pub fn delete_selection(&mut self) {
        if !self.selection.has_selection() {
            return;
        }

        let (start, end) = self.selection.ordered();
        let start_idx = self.buffer.line_col_to_char(start.line, start.col);
        let end_idx = self.buffer.line_col_to_char(end.line, end.col);

        self.buffer.delete_range(start_idx, end_idx);

        self.cursor = start;
        self.selection = Selection::new(self.cursor);
        self.modified = true;
    }

    /// Get the selected text
    pub fn selected_text(&self) -> String {
        if !self.selection.has_selection() {
            return String::new();
        }

        let (start, end) = self.selection.ordered();
        let start_idx = self.buffer.line_col_to_char(start.line, start.col);
        let end_idx = self.buffer.line_col_to_char(end.line, end.col);

        self.buffer.slice_to_string(start_idx, end_idx)
    }

    /// Move cursor right
    pub fn move_right(&mut self, extend_selection: bool) {
        let line_len = self.line_len(self.cursor.line);
        let total_lines = self.line_count();
        self.cursor.move_right(line_len, total_lines);

        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::new(self.cursor);
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self, extend_selection: bool) {
        self.cursor.move_left(|line| self.buffer.line_len(line));

        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::new(self.cursor);
        }
    }

    /// Move cursor up
    pub fn move_up(&mut self, extend_selection: bool) {
        self.cursor.move_up(|line| self.buffer.line_len(line));

        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::new(self.cursor);
        }
    }

    /// Move cursor down
    pub fn move_down(&mut self, extend_selection: bool) {
        let total_lines = self.line_count();
        self.cursor
            .move_down(total_lines, |line| self.buffer.line_len(line));

        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::new(self.cursor);
        }
    }

    /// Move cursor to line start
    pub fn move_to_line_start(&mut self, extend_selection: bool) {
        self.cursor.move_to_line_start();

        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::new(self.cursor);
        }
    }

    /// Move cursor to line end
    pub fn move_to_line_end(&mut self, extend_selection: bool) {
        let line_len = self.line_len(self.cursor.line);
        self.cursor.move_to_line_end(line_len);

        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::new(self.cursor);
        }
    }

    /// Move cursor to document start
    pub fn move_to_start(&mut self, extend_selection: bool) {
        self.cursor.move_to_start();

        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::new(self.cursor);
        }
    }

    /// Move cursor to document end
    pub fn move_to_end(&mut self, extend_selection: bool) {
        let total_lines = self.line_count();
        let last_line_len = self.line_len(total_lines.saturating_sub(1));
        self.cursor.move_to_end(total_lines, last_line_len);

        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::new(self.cursor);
        }
    }

    /// Move cursor to a specific line and column
    pub fn move_to(&mut self, line: usize, col: usize, extend_selection: bool) {
        let line = line.min(self.line_count().saturating_sub(1));
        let col = col.min(self.line_len(line));
        self.cursor.move_to(line, col);

        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::new(self.cursor);
        }
    }

    /// Page up
    pub fn page_up(&mut self, page_size: usize, extend_selection: bool) {
        if self.cursor.line > page_size {
            self.cursor.line -= page_size;
        } else {
            self.cursor.line = 0;
        }
        let line_len = self.line_len(self.cursor.line);
        self.cursor.col = self.cursor.wanted_col.min(line_len);

        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::new(self.cursor);
        }
    }

    /// Page down
    pub fn page_down(&mut self, page_size: usize, extend_selection: bool) {
        let total_lines = self.line_count();
        self.cursor.line = (self.cursor.line + page_size).min(total_lines.saturating_sub(1));
        let line_len = self.line_len(self.cursor.line);
        self.cursor.col = self.cursor.wanted_col.min(line_len);

        if extend_selection {
            self.selection.head = self.cursor;
        } else {
            self.selection = Selection::new(self.cursor);
        }
    }

    /// Select all text
    pub fn select_all(&mut self) {
        self.selection.anchor = Cursor::new();
        let total_lines = self.line_count();
        let last_line_len = self.line_len(total_lines.saturating_sub(1));
        self.cursor.move_to_end(total_lines, last_line_len);
        self.selection.head = self.cursor;
    }

    /// Ensure the cursor is visible in the viewport
    pub fn ensure_cursor_visible(&mut self, visible_lines: usize, visible_cols: usize) {
        // Vertical scrolling
        if self.cursor.line < self.scroll_y {
            self.scroll_y = self.cursor.line;
        } else if self.cursor.line >= self.scroll_y + visible_lines {
            self.scroll_y = self.cursor.line - visible_lines + 1;
        }

        // Horizontal scrolling
        if self.cursor.col < self.scroll_x {
            self.scroll_x = self.cursor.col;
        } else if self.cursor.col >= self.scroll_x + visible_cols {
            self.scroll_x = self.cursor.col - visible_cols + 1;
        }
    }

    /// Toggle insert/overwrite mode
    pub fn toggle_insert_mode(&mut self) {
        self.insert_mode = !self.insert_mode;
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect filetype from file extension
fn detect_filetype(path: &std::path::Path) -> String {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "rs" => "Rust",
        "py" => "Python",
        "js" => "JavaScript",
        "ts" => "TypeScript",
        "jsx" => "JavaScript (React)",
        "tsx" => "TypeScript (React)",
        "c" | "h" => "C",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "C++",
        "java" => "Java",
        "go" => "Go",
        "rb" => "Ruby",
        "php" => "PHP",
        "swift" => "Swift",
        "kt" | "kts" => "Kotlin",
        "scala" => "Scala",
        "cs" => "C#",
        "fs" | "fsx" => "F#",
        "hs" => "Haskell",
        "ml" | "mli" => "OCaml",
        "ex" | "exs" => "Elixir",
        "erl" | "hrl" => "Erlang",
        "clj" | "cljs" | "cljc" => "Clojure",
        "lua" => "Lua",
        "pl" | "pm" => "Perl",
        "r" => "R",
        "jl" => "Julia",
        "dart" => "Dart",
        "v" => "V",
        "zig" => "Zig",
        "nim" => "Nim",
        "cr" => "Crystal",
        "sh" | "bash" | "zsh" => "Shell",
        "ps1" => "PowerShell",
        "sql" => "SQL",
        "html" | "htm" => "HTML",
        "css" => "CSS",
        "scss" | "sass" => "SCSS",
        "less" => "Less",
        "json" => "JSON",
        "yaml" | "yml" => "YAML",
        "toml" => "TOML",
        "xml" => "XML",
        "md" | "markdown" => "Markdown",
        "rst" => "reStructuredText",
        "tex" => "LaTeX",
        "vim" => "Vim Script",
        "dockerfile" => "Dockerfile",
        "makefile" => "Makefile",
        "cmake" => "CMake",
        "gradle" => "Gradle",
        _ => "Plain Text",
    }
    .to_string()
}
