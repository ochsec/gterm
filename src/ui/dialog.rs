use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use std::path::{Path, PathBuf};

/// Represents an active dialog
#[derive(Debug, Clone)]
pub enum Dialog {
    /// File open dialog
    FileOpen(FileOpenDialog),
    /// File save as dialog
    FileSaveAs(FileSaveAsDialog),
    /// Message/alert dialog
    Message(MessageDialog),
    /// Go to line dialog
    GoToLine(GoToLineDialog),
}

/// File open dialog state
#[derive(Debug, Clone)]
pub struct FileOpenDialog {
    /// Current directory being browsed
    pub current_dir: PathBuf,
    /// List of entries in current directory
    pub entries: Vec<DirEntry>,
    /// Currently selected index
    pub selected: usize,
    /// Scroll offset
    pub scroll: usize,
    /// Text input for filename/path
    pub input: String,
    /// Whether input field is focused (vs file list)
    pub input_focused: bool,
}

/// A directory entry for the file browser
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

/// File save as dialog state
#[derive(Debug, Clone)]
pub struct FileSaveAsDialog {
    /// Current directory being browsed
    pub current_dir: PathBuf,
    /// List of entries in current directory
    pub entries: Vec<DirEntry>,
    /// Currently selected index in file list
    pub selected: usize,
    /// Filename input
    pub filename: String,
    /// Which field is focused: 0 = filename, 1 = file list
    pub focus: usize,
}

/// Simple message dialog
#[derive(Debug, Clone)]
pub struct MessageDialog {
    pub title: String,
    pub message: String,
}

/// Go to line dialog
#[derive(Debug, Clone)]
pub struct GoToLineDialog {
    /// Line number input
    pub input: String,
    /// Total lines in document (for display)
    pub total_lines: usize,
    /// Error message if invalid input
    pub error: Option<String>,
}

impl FileSaveAsDialog {
    /// Create a new file save as dialog starting at the given directory
    pub fn new(start_dir: PathBuf, initial_filename: String) -> Self {
        let mut dialog = Self {
            current_dir: start_dir,
            entries: Vec::new(),
            selected: 0,
            filename: initial_filename,
            focus: 0, // Start with filename focused
        };
        dialog.refresh_entries();
        dialog
    }

    /// Refresh the directory listing
    pub fn refresh_entries(&mut self) {
        self.entries.clear();

        // Add parent directory entry if not at root
        if let Some(parent) = self.current_dir.parent() {
            self.entries.push(DirEntry {
                name: "..".to_string(),
                path: parent.to_path_buf(),
                is_dir: true,
            });
        }

        // Read directory contents
        if let Ok(read_dir) = std::fs::read_dir(&self.current_dir) {
            let mut dirs: Vec<DirEntry> = Vec::new();
            let mut files: Vec<DirEntry> = Vec::new();

            for entry in read_dir.filter_map(|e| e.ok()) {
                let path = entry.path();
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                // Skip hidden files
                if name.starts_with('.') {
                    continue;
                }

                let is_dir = path.is_dir();
                let entry = DirEntry { name, path, is_dir };

                if is_dir {
                    dirs.push(entry);
                } else {
                    files.push(entry);
                }
            }

            // Sort alphabetically (case-insensitive)
            dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            // Add directories first, then files
            self.entries.extend(dirs);
            self.entries.extend(files);
        }

        // Reset selection
        self.selected = 0;
    }

    /// Move selection up in file list
    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down in file list
    pub fn move_down(&mut self) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
        }
    }

    /// Page up in file list
    pub fn page_up(&mut self, amount: usize) {
        self.selected = self.selected.saturating_sub(amount);
    }

    /// Page down in file list
    pub fn page_down(&mut self, amount: usize) {
        self.selected = (self.selected + amount).min(self.entries.len().saturating_sub(1));
    }

    /// Enter the selected directory or select the file
    pub fn enter_selected(&mut self) {
        if let Some(entry) = self.entries.get(self.selected) {
            if entry.is_dir {
                // Navigate into directory
                self.current_dir = entry.path.clone();
                self.refresh_entries();
            } else {
                // Select file - put its name in the filename field
                self.filename = entry.name.clone();
                self.focus = 0; // Focus back to filename
            }
        }
    }

    /// Go to parent directory
    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.refresh_entries();
        }
    }

    /// Handle text input for filename
    pub fn handle_input(&mut self, c: char) {
        self.filename.push(c);
    }

    /// Handle backspace in filename input
    pub fn handle_backspace(&mut self) {
        self.filename.pop();
    }

    /// Toggle focus between filename field and file list
    pub fn toggle_focus(&mut self) {
        self.focus = if self.focus == 0 { 1 } else { 0 };
    }

    /// Get the full path to save to
    pub fn get_save_path(&self) -> PathBuf {
        self.current_dir.join(&self.filename)
    }

    /// Check if filename is valid (not empty)
    pub fn is_valid(&self) -> bool {
        !self.filename.trim().is_empty()
    }

    /// Get the currently selected entry
    pub fn selected_entry(&self) -> Option<&DirEntry> {
        self.entries.get(self.selected)
    }
}

impl GoToLineDialog {
    /// Create a new go to line dialog
    pub fn new(total_lines: usize) -> Self {
        Self {
            input: String::new(),
            total_lines,
            error: None,
        }
    }

    /// Handle character input (only digits allowed)
    pub fn handle_input(&mut self, c: char) {
        if c.is_ascii_digit() {
            self.input.push(c);
            self.error = None;
        }
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self) {
        self.input.pop();
        self.error = None;
    }

    /// Parse and validate the line number, returns 0-indexed line if valid
    pub fn get_line_number(&mut self) -> Option<usize> {
        if self.input.is_empty() {
            self.error = Some("Enter a line number".to_string());
            return None;
        }

        match self.input.parse::<usize>() {
            Ok(line) if line >= 1 && line <= self.total_lines => {
                Some(line - 1) // Convert to 0-indexed
            }
            Ok(line) if line == 0 => {
                self.error = Some("Line number must be at least 1".to_string());
                None
            }
            Ok(_) => {
                self.error = Some(format!("Line must be 1-{}", self.total_lines));
                None
            }
            Err(_) => {
                self.error = Some("Invalid number".to_string());
                None
            }
        }
    }
}

impl FileOpenDialog {
    /// Create a new file open dialog starting at the given directory
    pub fn new(start_dir: PathBuf) -> Self {
        let mut dialog = Self {
            current_dir: start_dir,
            entries: Vec::new(),
            selected: 0,
            scroll: 0,
            input: String::new(),
            input_focused: false,
        };
        dialog.refresh_entries();
        dialog
    }

    /// Refresh the directory listing
    pub fn refresh_entries(&mut self) {
        self.entries.clear();

        // Add parent directory entry if not at root
        if let Some(parent) = self.current_dir.parent() {
            self.entries.push(DirEntry {
                name: "..".to_string(),
                path: parent.to_path_buf(),
                is_dir: true,
            });
        }

        // Read directory contents
        if let Ok(read_dir) = std::fs::read_dir(&self.current_dir) {
            let mut dirs: Vec<DirEntry> = Vec::new();
            let mut files: Vec<DirEntry> = Vec::new();

            for entry in read_dir.filter_map(|e| e.ok()) {
                let path = entry.path();
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                // Skip hidden files
                if name.starts_with('.') {
                    continue;
                }

                let is_dir = path.is_dir();
                let entry = DirEntry { name, path, is_dir };

                if is_dir {
                    dirs.push(entry);
                } else {
                    files.push(entry);
                }
            }

            // Sort alphabetically (case-insensitive)
            dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            // Add directories first, then files
            self.entries.extend(dirs);
            self.entries.extend(files);
        }

        // Reset selection
        self.selected = 0;
        self.scroll = 0;
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
        }
    }

    /// Page up
    pub fn page_up(&mut self, amount: usize) {
        self.selected = self.selected.saturating_sub(amount);
    }

    /// Page down
    pub fn page_down(&mut self, amount: usize) {
        self.selected = (self.selected + amount).min(self.entries.len().saturating_sub(1));
    }

    /// Enter the selected directory or return the selected file
    pub fn enter_selected(&mut self) -> Option<PathBuf> {
        if let Some(entry) = self.entries.get(self.selected) {
            if entry.is_dir {
                // Navigate into directory
                self.current_dir = entry.path.clone();
                self.refresh_entries();
                self.input = self.current_dir.to_string_lossy().to_string();
                None
            } else {
                // Return selected file
                Some(entry.path.clone())
            }
        } else {
            None
        }
    }

    /// Go to parent directory
    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.refresh_entries();
            self.input = self.current_dir.to_string_lossy().to_string();
        }
    }

    /// Handle text input
    pub fn handle_input(&mut self, c: char) {
        self.input.push(c);
    }

    /// Handle backspace in input
    pub fn handle_backspace(&mut self) {
        self.input.pop();
    }

    /// Try to navigate to the path in the input field
    pub fn navigate_to_input(&mut self) -> Option<PathBuf> {
        let path = PathBuf::from(&self.input);
        if path.is_dir() {
            self.current_dir = path;
            self.refresh_entries();
            None
        } else if path.is_file() {
            Some(path)
        } else {
            None
        }
    }

    /// Toggle focus between input and file list
    pub fn toggle_focus(&mut self) {
        self.input_focused = !self.input_focused;
    }

    /// Get the currently selected entry
    pub fn selected_entry(&self) -> Option<&DirEntry> {
        self.entries.get(self.selected)
    }
}

/// Draw a file open dialog
pub fn draw_file_open_dialog(frame: &mut Frame, app: &App, dialog: &FileOpenDialog) {
    let area = frame.area();

    // Dialog size: 60% width, 70% height, centered
    let dialog_width = (area.width as f32 * 0.6) as u16;
    let dialog_height = (area.height as f32 * 0.7) as u16;
    let dialog_x = (area.width - dialog_width) / 2;
    let dialog_y = (area.height - dialog_height) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear area behind dialog
    frame.render_widget(Clear, dialog_area);

    // Draw dialog border
    let block = Block::default()
        .title(" Open File ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.border_focused))
        .style(Style::default().bg(app.theme.sidebar_bg));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    // Split inner area: path input (2 lines), file list (rest), help (1 line)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Path input with border
            Constraint::Min(5),    // File list
            Constraint::Length(1), // Help text
        ])
        .split(inner);

    // Draw path input
    let input_style = if dialog.input_focused {
        Style::default().fg(app.theme.fg).bg(app.theme.editor_bg)
    } else {
        Style::default()
            .fg(app.theme.line_number)
            .bg(app.theme.editor_bg)
    };

    let input_block = Block::default()
        .title(" Path ")
        .borders(Borders::ALL)
        .border_style(if dialog.input_focused {
            Style::default().fg(app.theme.border_focused)
        } else {
            Style::default().fg(app.theme.border)
        });

    let input_text = if dialog.input_focused {
        format!("{}▏", dialog.input)
    } else {
        dialog.current_dir.to_string_lossy().to_string()
    };

    let input = Paragraph::new(input_text)
        .style(input_style)
        .block(input_block);

    frame.render_widget(input, chunks[0]);

    // Draw file list
    let list_items: Vec<ListItem> = dialog
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let icon = if entry.is_dir { "📁 " } else { "📄 " };
            let name = format!("{}{}", icon, entry.name);

            let style = if i == dialog.selected && !dialog.input_focused {
                Style::default()
                    .fg(app.theme.menubar_bg)
                    .bg(app.theme.statusbar_bg)
            } else if entry.is_dir {
                Style::default().fg(app.theme.tree_dir)
            } else {
                Style::default().fg(app.theme.tree_file)
            };

            ListItem::new(name).style(style)
        })
        .collect();

    let list = List::new(list_items)
        .block(
            Block::default()
                .title(format!(" {} ", dialog.current_dir.to_string_lossy()))
                .borders(Borders::ALL)
                .border_style(if !dialog.input_focused {
                    Style::default().fg(app.theme.border_focused)
                } else {
                    Style::default().fg(app.theme.border)
                }),
        )
        .style(Style::default().bg(app.theme.sidebar_bg));

    frame.render_widget(list, chunks[1]);

    // Draw help text
    let help_text = "↑↓:Navigate  Enter:Select  Tab:Switch focus  Esc:Cancel  Backspace:Go up";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(app.theme.line_number))
        .alignment(Alignment::Center);

    frame.render_widget(help, chunks[2]);
}

/// Draw a file save as dialog
pub fn draw_file_save_as_dialog(frame: &mut Frame, app: &App, dialog: &FileSaveAsDialog) {
    let area = frame.area();

    // Dialog size: 60% width, 70% height, centered
    let dialog_width = (area.width as f32 * 0.6) as u16;
    let dialog_height = (area.height as f32 * 0.7) as u16;
    let dialog_x = (area.width - dialog_width) / 2;
    let dialog_y = (area.height - dialog_height) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear area behind dialog
    frame.render_widget(Clear, dialog_area);

    // Draw dialog border
    let block = Block::default()
        .title(" Save As ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.border_focused))
        .style(Style::default().bg(app.theme.sidebar_bg));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    // Split inner area: filename input (3 lines), file list (rest), help (1 line)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Filename input with border
            Constraint::Min(5),    // File list
            Constraint::Length(1), // Help text
        ])
        .split(inner);

    // Draw filename input
    let input_style = if dialog.focus == 0 {
        Style::default().fg(app.theme.fg).bg(app.theme.editor_bg)
    } else {
        Style::default()
            .fg(app.theme.line_number)
            .bg(app.theme.editor_bg)
    };

    let input_block = Block::default()
        .title(" Filename ")
        .borders(Borders::ALL)
        .border_style(if dialog.focus == 0 {
            Style::default().fg(app.theme.border_focused)
        } else {
            Style::default().fg(app.theme.border)
        });

    let input_text = if dialog.focus == 0 {
        format!("{}▏", dialog.filename)
    } else {
        dialog.filename.clone()
    };

    let input = Paragraph::new(input_text)
        .style(input_style)
        .block(input_block);

    frame.render_widget(input, chunks[0]);

    // Draw file list
    let list_items: Vec<ListItem> = dialog
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let icon = if entry.is_dir { "📁 " } else { "📄 " };
            let name = format!("{}{}", icon, entry.name);

            let style = if i == dialog.selected && dialog.focus == 1 {
                Style::default()
                    .fg(app.theme.menubar_bg)
                    .bg(app.theme.statusbar_bg)
            } else if entry.is_dir {
                Style::default().fg(app.theme.tree_dir)
            } else {
                Style::default().fg(app.theme.tree_file)
            };

            ListItem::new(name).style(style)
        })
        .collect();

    let list = List::new(list_items)
        .block(
            Block::default()
                .title(format!(" {} ", dialog.current_dir.to_string_lossy()))
                .borders(Borders::ALL)
                .border_style(if dialog.focus == 1 {
                    Style::default().fg(app.theme.border_focused)
                } else {
                    Style::default().fg(app.theme.border)
                }),
        )
        .style(Style::default().bg(app.theme.sidebar_bg));

    frame.render_widget(list, chunks[1]);

    // Draw help text
    let help_text = "Tab:Switch focus  Enter:Save/Select  ↑↓:Navigate  Backspace:Go up  Esc:Cancel";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(app.theme.line_number))
        .alignment(Alignment::Center);

    frame.render_widget(help, chunks[2]);
}

/// Draw a message dialog
pub fn draw_message_dialog(frame: &mut Frame, app: &App, dialog: &MessageDialog) {
    let area = frame.area();

    // Dialog size based on content
    let dialog_width = (dialog.message.len() as u16 + 6)
        .max(30)
        .min(area.width - 4);
    let dialog_height = 5;
    let dialog_x = (area.width - dialog_width) / 2;
    let dialog_y = (area.height - dialog_height) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear area behind dialog
    frame.render_widget(Clear, dialog_area);

    // Draw dialog
    let block = Block::default()
        .title(format!(" {} ", dialog.title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.border_focused))
        .style(Style::default().bg(app.theme.sidebar_bg));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let message = Paragraph::new(dialog.message.as_str())
        .style(Style::default().fg(app.theme.fg))
        .alignment(Alignment::Center);

    frame.render_widget(message, inner);
}

/// Draw a go to line dialog
pub fn draw_go_to_line_dialog(frame: &mut Frame, app: &App, dialog: &GoToLineDialog) {
    let area = frame.area();

    // Small dialog centered on screen
    let dialog_width = 40u16.min(area.width - 4);
    let dialog_height = 6u16;
    let dialog_x = (area.width - dialog_width) / 2;
    let dialog_y = (area.height - dialog_height) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear area behind dialog
    frame.render_widget(Clear, dialog_area);

    // Draw dialog border
    let block = Block::default()
        .title(" Go to Line ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.border_focused))
        .style(Style::default().bg(app.theme.sidebar_bg));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    // Split inner area: input (1 line), info/error (1 line), help (1 line)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Input
            Constraint::Length(1), // Info/error
            Constraint::Length(1), // Help
        ])
        .split(inner);

    // Draw input field
    let input_text = format!("Line: {}|", dialog.input);
    let input =
        Paragraph::new(input_text).style(Style::default().fg(app.theme.fg).bg(app.theme.editor_bg));
    frame.render_widget(input, chunks[0]);

    // Draw info or error
    let info_text = if let Some(ref error) = dialog.error {
        Paragraph::new(error.as_str()).style(Style::default().fg(Color::Red))
    } else {
        Paragraph::new(format!("(1-{})", dialog.total_lines))
            .style(Style::default().fg(app.theme.line_number))
    };
    frame.render_widget(info_text, chunks[1]);

    // Draw help
    let help = Paragraph::new("Enter: Go  Esc: Cancel")
        .style(Style::default().fg(app.theme.line_number))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}

/// Draw the active dialog (if any)
pub fn draw_dialog(frame: &mut Frame, app: &App) {
    if let Some(dialog) = &app.dialog {
        match dialog {
            Dialog::FileOpen(d) => draw_file_open_dialog(frame, app, d),
            Dialog::FileSaveAs(d) => draw_file_save_as_dialog(frame, app, d),
            Dialog::Message(d) => draw_message_dialog(frame, app, d),
            Dialog::GoToLine(d) => draw_go_to_line_dialog(frame, app, d),
        }
    }
}
