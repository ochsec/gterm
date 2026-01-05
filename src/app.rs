use crate::editor::Document;
use crate::file_tree::FileTree;
use crate::highlighting::HighlightingManager;
use crate::input::InputHandler;
use crate::search::SearchState;
use crate::terminal::Terminal;
use crate::theme::Theme;
use crate::ui::dialog::{Dialog, FileOpenDialog, FileSaveAsDialog};
use crate::ui::{self, Pane};
use crate::utils::clipboard::Clipboard;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::prelude::*;
use std::path::PathBuf;
use std::time::Duration;

/// The main application state
pub struct App {
    /// Whether the application should quit
    pub should_quit: bool,
    /// Currently focused pane
    pub focused_pane: Pane,
    /// Whether the sidebar is visible
    pub show_sidebar: bool,
    /// Whether the editor pane is visible
    pub show_editor: bool,
    /// Whether the terminal pane is visible
    pub show_terminal: bool,
    /// Sidebar width as percentage (0-100)
    pub sidebar_width_percent: u16,
    /// Terminal height as percentage (0-100)
    pub terminal_height_percent: u16,
    /// Whether we're currently resizing a pane
    pub resizing: Option<ResizeTarget>,
    /// The color theme
    pub theme: Theme,
    /// Input handler
    pub input_handler: InputHandler,
    /// Current working directory for file tree
    pub cwd: std::path::PathBuf,
    /// File tree state
    pub file_tree: FileTree,
    /// Last known file tree area for mouse hit detection
    pub file_tree_area: Option<Rect>,
    /// Open documents
    pub documents: Vec<Document>,
    /// Currently active document index
    pub active_doc: usize,
    /// Last known editor area for mouse hit detection
    pub editor_area: Option<Rect>,
    /// Currently open menu (None = menu bar closed)
    pub menu_open: Option<usize>,
    /// Currently selected menu item within open menu
    pub menu_selected: Option<usize>,
    /// Menu positions for click detection (start_x, end_x, menu_index)
    pub menu_positions: Vec<(u16, u16, usize)>,
    /// Active dialog (if any)
    pub dialog: Option<Dialog>,
    /// Terminal emulator
    pub terminal: Option<Terminal>,
    /// Last known terminal area for resize detection
    pub terminal_area: Option<Rect>,
    /// System clipboard
    pub clipboard: Clipboard,
    /// Syntax highlighting manager
    pub highlighting: HighlightingManager,
    /// Search state
    pub search: SearchState,
}

/// Which divider is being resized
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeTarget {
    /// Resizing the sidebar width (vertical divider)
    Sidebar,
    /// Resizing the terminal height (horizontal divider)
    Terminal,
}

impl App {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let file_tree = FileTree::new(cwd.clone(), false);

        // Start with one empty document
        let documents = vec![Document::new()];

        Self {
            should_quit: false,
            focused_pane: Pane::Editor,
            show_sidebar: true,
            show_editor: true,
            show_terminal: true,
            sidebar_width_percent: 20,
            terminal_height_percent: 50,
            resizing: None,
            theme: Theme::dark(),
            input_handler: InputHandler::new(),
            cwd,
            file_tree,
            file_tree_area: None,
            documents,
            active_doc: 0,
            editor_area: None,
            menu_open: None,
            menu_selected: None,
            menu_positions: Vec::new(),
            dialog: None,
            terminal: Terminal::new(80, 24).ok(),
            terminal_area: None,
            clipboard: Clipboard::new(),
            highlighting: HighlightingManager::new(),
            search: SearchState::new(),
        }
    }

    /// Open the file open dialog
    pub fn show_open_dialog(&mut self) {
        let start_dir = self.cwd.clone();
        self.dialog = Some(Dialog::FileOpen(FileOpenDialog::new(start_dir)));
    }

    /// Open the file save as dialog
    pub fn show_save_as_dialog(&mut self) {
        let start_dir = if let Some(doc) = self.active_document() {
            // If document has a path, start in its directory
            doc.path
                .as_ref()
                .and_then(|p| p.parent())
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| self.cwd.clone())
        } else {
            self.cwd.clone()
        };

        // Get initial filename from document
        let initial_filename = if let Some(doc) = self.active_document() {
            doc.path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| doc.title().to_string())
        } else {
            "untitled.txt".to_string()
        };

        self.dialog = Some(Dialog::FileSaveAs(FileSaveAsDialog::new(
            start_dir,
            initial_filename,
        )));
    }

    /// Close any open dialog
    pub fn close_dialog(&mut self) {
        self.dialog = None;
    }

    /// Check if a dialog is open
    pub fn has_dialog(&self) -> bool {
        self.dialog.is_some()
    }

    /// Close the menu
    pub fn close_menu(&mut self) {
        self.menu_open = None;
        self.menu_selected = None;
    }

    /// Open a specific menu
    pub fn open_menu(&mut self, menu_idx: usize) {
        self.menu_open = Some(menu_idx);
        self.menu_selected = Some(0);
    }

    /// Move to the next menu
    pub fn next_menu(&mut self) {
        if let Some(idx) = self.menu_open {
            let next = (idx + 1) % crate::ui::menu_bar::MENUS.len();
            self.menu_open = Some(next);
            self.menu_selected = Some(0);
        }
    }

    /// Move to the previous menu
    pub fn prev_menu(&mut self) {
        if let Some(idx) = self.menu_open {
            let prev = if idx == 0 {
                crate::ui::menu_bar::MENUS.len() - 1
            } else {
                idx - 1
            };
            self.menu_open = Some(prev);
            self.menu_selected = Some(0);
        }
    }

    /// Move selection down in the menu
    pub fn menu_select_next(&mut self) {
        if let (Some(menu_idx), Some(sel)) = (self.menu_open, self.menu_selected) {
            if let Some((_, items)) = crate::ui::menu_bar::MENUS.get(menu_idx) {
                let mut next = sel + 1;
                // Skip separators and wrap
                while next < items.len() {
                    if items[next].action != crate::ui::menu_bar::MenuAction::Separator {
                        break;
                    }
                    next += 1;
                }
                if next >= items.len() {
                    next = 0;
                    // Skip initial separators
                    while next < items.len()
                        && items[next].action == crate::ui::menu_bar::MenuAction::Separator
                    {
                        next += 1;
                    }
                }
                self.menu_selected = Some(next);
            }
        }
    }

    /// Move selection up in the menu
    pub fn menu_select_prev(&mut self) {
        if let (Some(menu_idx), Some(sel)) = (self.menu_open, self.menu_selected) {
            if let Some((_, items)) = crate::ui::menu_bar::MENUS.get(menu_idx) {
                let mut prev = if sel == 0 { items.len() - 1 } else { sel - 1 };
                // Skip separators
                while prev > 0 && items[prev].action == crate::ui::menu_bar::MenuAction::Separator {
                    prev -= 1;
                }
                // If we hit a separator at 0, go to end
                if items[prev].action == crate::ui::menu_bar::MenuAction::Separator {
                    prev = items.len() - 1;
                    while prev > 0
                        && items[prev].action == crate::ui::menu_bar::MenuAction::Separator
                    {
                        prev -= 1;
                    }
                }
                self.menu_selected = Some(prev);
            }
        }
    }

    /// Execute the currently selected menu action
    pub fn execute_menu_action(&mut self) {
        use crate::ui::menu_bar::MenuAction;

        if let (Some(menu_idx), Some(sel)) = (self.menu_open, self.menu_selected) {
            if let Some((_, items)) = crate::ui::menu_bar::MENUS.get(menu_idx) {
                if let Some(item) = items.get(sel) {
                    let action = item.action;
                    self.close_menu();

                    match action {
                        MenuAction::NewFile => self.new_file(),
                        MenuAction::OpenFile => {
                            self.show_open_dialog();
                        }
                        MenuAction::Save => {
                            if let Some(doc) = self.active_document_mut() {
                                if doc.path.is_some() {
                                    let _ = doc.save();
                                }
                            }
                        }
                        MenuAction::SaveAs => {
                            self.show_save_as_dialog();
                        }
                        MenuAction::SaveAll => {
                            for doc in &mut self.documents {
                                if doc.path.is_some() && doc.modified {
                                    let _ = doc.save();
                                }
                            }
                        }
                        MenuAction::Close => self.close_current(),
                        MenuAction::CloseAll => {
                            self.documents.clear();
                            self.documents.push(Document::new());
                            self.active_doc = 0;
                        }
                        MenuAction::Quit => self.should_quit = true,

                        MenuAction::Undo => {
                            // TODO: Implement undo
                        }
                        MenuAction::Redo => {
                            // TODO: Implement redo
                        }
                        MenuAction::Cut => {
                            // Get selected text first
                            let text = self.active_document().map(|doc| doc.selected_text());
                            if let Some(text) = text {
                                if !text.is_empty() {
                                    let _ = self.clipboard.set_text(&text);
                                    if let Some(doc) = self.active_document_mut() {
                                        doc.delete_selection();
                                    }
                                }
                            }
                        }
                        MenuAction::Copy => {
                            if let Some(doc) = self.active_document() {
                                let text = doc.selected_text();
                                if !text.is_empty() {
                                    let _ = self.clipboard.set_text(&text);
                                }
                            }
                        }
                        MenuAction::Paste => {
                            if let Ok(text) = self.clipboard.get_text() {
                                if let Some(doc) = self.active_document_mut() {
                                    doc.insert_str(&text);
                                }
                            }
                        }
                        MenuAction::SelectAll => {
                            if let Some(doc) = self.active_document_mut() {
                                doc.select_all();
                            }
                        }

                        MenuAction::Find => {
                            self.search.open();
                            self.focused_pane = Pane::Editor;
                        }
                        MenuAction::FindNext => {
                            self.find_next();
                        }
                        MenuAction::FindPrevious => {
                            self.find_prev();
                        }
                        MenuAction::Replace => {
                            self.search.open_replace();
                            self.focused_pane = Pane::Editor;
                        }
                        MenuAction::GoToLine => {
                            // TODO: Open go to line dialog
                        }

                        MenuAction::ToggleSidebar => {
                            self.show_sidebar = !self.show_sidebar;
                        }
                        MenuAction::ToggleEditor => {
                            self.show_editor = !self.show_editor;
                        }
                        MenuAction::ToggleTerminal => {
                            self.show_terminal = !self.show_terminal;
                        }
                        MenuAction::FocusEditor => {
                            self.focused_pane = Pane::Editor;
                        }
                        MenuAction::FocusFileTree => {
                            if self.show_sidebar {
                                self.focused_pane = Pane::FileTree;
                            }
                        }
                        MenuAction::FocusTerminal => {
                            if self.show_terminal {
                                self.focused_pane = Pane::Terminal;
                            }
                        }

                        MenuAction::About => {
                            // TODO: Show about dialog
                        }

                        MenuAction::Separator => {}
                    }
                }
            }
        }
    }

    /// Get the current active document
    pub fn active_document(&self) -> Option<&Document> {
        self.documents.get(self.active_doc)
    }

    /// Get the current active document mutably
    pub fn active_document_mut(&mut self) -> Option<&mut Document> {
        self.documents.get_mut(self.active_doc)
    }

    /// Open a file in a new tab
    pub fn open_file(&mut self, path: PathBuf) -> Result<()> {
        // Check if file is already open
        for (i, doc) in self.documents.iter().enumerate() {
            if doc.path.as_ref() == Some(&path) {
                self.active_doc = i;
                return Ok(());
            }
        }

        // Open the file
        let doc = Document::open(path)?;
        self.documents.push(doc);
        self.active_doc = self.documents.len() - 1;
        Ok(())
    }

    /// Create a new empty document
    pub fn new_file(&mut self) {
        self.documents.push(Document::new());
        self.active_doc = self.documents.len() - 1;
    }

    /// Close the current document
    pub fn close_current(&mut self) {
        if self.documents.len() > 1 {
            self.documents.remove(self.active_doc);
            if self.active_doc >= self.documents.len() {
                self.active_doc = self.documents.len() - 1;
            }
        } else {
            // Replace with empty document instead of removing last one
            self.documents[0] = Document::new();
        }
    }

    /// Switch to the next tab
    pub fn next_tab(&mut self) {
        if !self.documents.is_empty() {
            self.active_doc = (self.active_doc + 1) % self.documents.len();
        }
    }

    /// Switch to the previous tab
    pub fn prev_tab(&mut self) {
        if !self.documents.is_empty() {
            self.active_doc = (self.active_doc + self.documents.len() - 1) % self.documents.len();
        }
    }

    /// Switch to a specific tab by index (1-based for user, converted to 0-based)
    pub fn go_to_tab(&mut self, tab: u8) {
        let index = if tab == 0 {
            self.documents.len().saturating_sub(1) // Tab 0 = last tab
        } else {
            (tab as usize).saturating_sub(1)
        };
        if index < self.documents.len() {
            self.active_doc = index;
        }
    }

    /// Perform search with current query in active document
    pub fn do_search(&mut self) {
        // Clone the document to avoid borrow issues
        let doc_clone = match self.documents.get(self.active_doc) {
            Some(doc) => doc.clone(),
            None => return,
        };
        self.search.search(&doc_clone);
    }

    /// Find the next search match and move cursor to it
    pub fn find_next(&mut self) {
        if let Some(doc) = self.active_document() {
            let (line, col) = (doc.cursor.line, doc.cursor.col);
            if let Some(m) = self.search.find_next_from(line, col) {
                if let Some(doc) = self.active_document_mut() {
                    doc.move_to(m.line, m.start_col, false);
                    // Select the match
                    doc.selection.anchor = doc.cursor;
                    doc.cursor.col = m.end_col;
                    doc.selection.head = doc.cursor;
                    // Scroll to make cursor visible (use reasonable defaults)
                    doc.ensure_cursor_visible(30, 80);
                }
            }
        }
    }

    /// Find the previous search match and move cursor to it
    pub fn find_prev(&mut self) {
        if let Some(doc) = self.active_document() {
            let (line, col) = (doc.cursor.line, doc.cursor.col);
            if let Some(m) = self.search.find_prev_from(line, col) {
                if let Some(doc) = self.active_document_mut() {
                    doc.move_to(m.line, m.start_col, false);
                    // Select the match
                    doc.selection.anchor = doc.cursor;
                    doc.cursor.col = m.end_col;
                    doc.selection.head = doc.cursor;
                    // Scroll to make cursor visible (use reasonable defaults)
                    doc.ensure_cursor_visible(30, 80);
                }
            }
        }
    }

    /// Run the main application loop
    pub fn run(&mut self, ratatui_terminal: &mut ratatui::Terminal<impl Backend>) -> Result<()> {
        while !self.should_quit {
            // Read PTY output before drawing
            if let Some(ref mut term) = self.terminal {
                let _ = term.read_output();
            }

            // Draw UI - we need to use a raw pointer trick since terminal.draw()
            // takes a closure and we need &mut self
            let app_ptr = self as *mut App;
            ratatui_terminal.draw(|frame| {
                // SAFETY: We have exclusive access to self during this closure
                // and the closure doesn't escape
                unsafe {
                    ui::draw(frame, &mut *app_ptr);
                }
            })?;

            // Check if terminal area changed and resize PTY
            self.check_terminal_resize();

            // Handle events with a small timeout for responsiveness
            if event::poll(Duration::from_millis(16))? {
                let event = event::read()?;
                self.handle_event(event)?;
            }
        }

        Ok(())
    }

    /// Check if terminal area changed and resize PTY accordingly
    fn check_terminal_resize(&mut self) {
        if let (Some(area), Some(ref mut term)) = (self.terminal_area, &mut self.terminal) {
            // Account for border (1 row for top border)
            let inner_cols = area.width.saturating_sub(0);
            let inner_rows = area.height.saturating_sub(1);

            if inner_cols != term.cols || inner_rows != term.rows {
                if inner_cols > 0 && inner_rows > 0 {
                    let _ = term.resize(inner_cols, inner_rows);
                }
            }
        }
    }

    /// Handle an input event
    fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key) => self.handle_key_event(key),
            Event::Mouse(mouse) => self.handle_mouse_event(mouse),
            Event::Resize(_, _) => {
                // Terminal resize is handled automatically by ratatui
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Handle keyboard events
    fn handle_key_event(&mut self, key: event::KeyEvent) -> Result<()> {
        // If dialog is open, handle dialog input first
        if self.dialog.is_some() {
            return self.handle_dialog_key(key);
        }

        // If search is active, handle search input
        if self.search.active {
            return self.handle_search_key(key);
        }

        // If menu is open, handle menu navigation first
        if self.menu_open.is_some() {
            match key.code {
                KeyCode::Esc => {
                    self.close_menu();
                    return Ok(());
                }
                KeyCode::Left => {
                    self.prev_menu();
                    return Ok(());
                }
                KeyCode::Right => {
                    self.next_menu();
                    return Ok(());
                }
                KeyCode::Up => {
                    self.menu_select_prev();
                    return Ok(());
                }
                KeyCode::Down => {
                    self.menu_select_next();
                    return Ok(());
                }
                KeyCode::Enter => {
                    self.execute_menu_action();
                    return Ok(());
                }
                _ => {
                    // Close menu on any other key
                    self.close_menu();
                }
            }
        }

        // F10 or Alt to open menu
        if key.code == KeyCode::F(10)
            || (key.modifiers == KeyModifiers::ALT && key.code == KeyCode::Char('f'))
        {
            self.open_menu(0); // Open File menu
            return Ok(());
        }

        // Global shortcuts (work regardless of focus)
        // Note: Handle Ctrl+Shift combos by checking for both uppercase letter (when shift works)
        // and checking for Shift modifier with lowercase (for terminals that report it differently)
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let ctrl_shift = ctrl && shift;

        match (key.modifiers, key.code) {
            // Quit: Ctrl+Q
            (KeyModifiers::CONTROL, KeyCode::Char('q')) => {
                self.should_quit = true;
            }
            // Toggle sidebar: Ctrl+B
            (KeyModifiers::CONTROL, KeyCode::Char('b')) => {
                self.show_sidebar = !self.show_sidebar;
            }
            // Toggle terminal: Ctrl+T
            (KeyModifiers::CONTROL, KeyCode::Char('t')) => {
                self.show_terminal = !self.show_terminal;
            }
            // Toggle editor: Ctrl+E
            (KeyModifiers::CONTROL, KeyCode::Char('e')) => {
                self.show_editor = !self.show_editor;
            }
            // Focus file tree: F3
            (KeyModifiers::NONE, KeyCode::F(3)) => {
                if self.show_sidebar {
                    self.focused_pane = Pane::FileTree;
                }
            }
            // Focus editor: F2
            (KeyModifiers::NONE, KeyCode::F(2)) => {
                self.focused_pane = Pane::Editor;
            }
            // Focus terminal: F4
            (KeyModifiers::NONE, KeyCode::F(4)) => {
                if self.show_terminal {
                    self.focused_pane = Pane::Terminal;
                }
            }
            // Tab to cycle focus (only when menu closed and not in editor)
            (KeyModifiers::NONE, KeyCode::Tab) if self.focused_pane != Pane::Editor => {
                self.cycle_focus(true);
            }
            (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                self.cycle_focus(false);
            }
            _ => {
                // Pass to focused pane handler
                self.handle_pane_key_event(key)?;
            }
        }

        Ok(())
    }

    /// Handle keyboard events for dialogs
    fn handle_dialog_key(&mut self, key: event::KeyEvent) -> Result<()> {
        let dialog = match &mut self.dialog {
            Some(d) => d,
            None => return Ok(()),
        };

        match dialog {
            Dialog::FileOpen(ref mut file_dialog) => {
                match key.code {
                    KeyCode::Esc => {
                        self.dialog = None;
                    }
                    KeyCode::Tab => {
                        file_dialog.toggle_focus();
                    }
                    KeyCode::Up if !file_dialog.input_focused => {
                        file_dialog.move_up();
                    }
                    KeyCode::Down if !file_dialog.input_focused => {
                        file_dialog.move_down();
                    }
                    KeyCode::PageUp if !file_dialog.input_focused => {
                        file_dialog.page_up(10);
                    }
                    KeyCode::PageDown if !file_dialog.input_focused => {
                        file_dialog.page_down(10);
                    }
                    KeyCode::Backspace => {
                        if file_dialog.input_focused {
                            file_dialog.handle_backspace();
                        } else {
                            file_dialog.go_up();
                        }
                    }
                    KeyCode::Enter => {
                        if file_dialog.input_focused {
                            // Try to navigate to input path
                            if let Some(path) = file_dialog.navigate_to_input() {
                                let _ = self.open_file(path);
                                self.dialog = None;
                            }
                        } else {
                            // Select current item
                            if let Some(path) = file_dialog.enter_selected() {
                                let _ = self.open_file(path);
                                self.dialog = None;
                            }
                        }
                    }
                    KeyCode::Char(c) if file_dialog.input_focused => {
                        file_dialog.handle_input(c);
                    }
                    _ => {}
                }
            }
            Dialog::FileSaveAs(ref mut save_dialog) => {
                match key.code {
                    KeyCode::Esc => {
                        self.dialog = None;
                    }
                    KeyCode::Tab => {
                        save_dialog.toggle_focus();
                    }
                    KeyCode::Up if save_dialog.focus == 1 => {
                        save_dialog.move_up();
                    }
                    KeyCode::Down if save_dialog.focus == 1 => {
                        save_dialog.move_down();
                    }
                    KeyCode::PageUp if save_dialog.focus == 1 => {
                        save_dialog.page_up(10);
                    }
                    KeyCode::PageDown if save_dialog.focus == 1 => {
                        save_dialog.page_down(10);
                    }
                    KeyCode::Backspace => {
                        if save_dialog.focus == 0 {
                            save_dialog.handle_backspace();
                        } else {
                            save_dialog.go_up();
                        }
                    }
                    KeyCode::Enter => {
                        if save_dialog.focus == 0 {
                            // Save with current filename
                            if save_dialog.is_valid() {
                                let save_path = save_dialog.get_save_path();
                                if let Some(doc) = self.active_document_mut() {
                                    doc.path = Some(save_path);
                                    let _ = doc.save();
                                }
                                // Refresh file tree to show the new file
                                self.file_tree.refresh();
                                self.dialog = None;
                            }
                        } else {
                            // Enter in file list: navigate into dir or select file
                            save_dialog.enter_selected();
                        }
                    }
                    KeyCode::Char(c) if save_dialog.focus == 0 => {
                        save_dialog.handle_input(c);
                    }
                    _ => {}
                }
            }
            Dialog::Message(_) => {
                // Any key closes message dialog
                self.dialog = None;
            }
        }

        Ok(())
    }

    /// Handle keyboard events for the search bar
    fn handle_search_key(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.search.close();
            }
            KeyCode::Enter => {
                // Find next match
                self.find_next();
            }
            KeyCode::Backspace => {
                self.search.backspace();
                self.do_search();
            }
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Handle Ctrl+G for next, Ctrl+Shift+G for prev
                    match c {
                        'g' => self.find_next(),
                        'G' => self.find_prev(),
                        'f' => {
                            // Ctrl+F while search is open - just keep it open
                        }
                        _ => {}
                    }
                } else {
                    self.search.input_char(c);
                    self.do_search();
                }
            }
            KeyCode::Up | KeyCode::F(3) if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.find_prev();
            }
            KeyCode::Down | KeyCode::F(3) => {
                self.find_next();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle key events for the currently focused pane
    fn handle_pane_key_event(&mut self, key: event::KeyEvent) -> Result<()> {
        match self.focused_pane {
            Pane::FileTree => {
                self.handle_file_tree_key(key)?;
            }
            Pane::Editor => {
                self.handle_editor_key(key)?;
            }
            Pane::Terminal => {
                self.handle_terminal_key(key)?;
            }
        }
        Ok(())
    }

    /// Handle keyboard events for the editor
    fn handle_editor_key(&mut self, key: event::KeyEvent) -> Result<()> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let alt = key.modifiers.contains(KeyModifiers::ALT);

        // Handle global editor shortcuts first
        match (ctrl, shift, key.code) {
            // New file: Ctrl+N
            (true, false, KeyCode::Char('n')) => {
                self.new_file();
                return Ok(());
            }
            // Open file: Ctrl+O
            (true, false, KeyCode::Char('o')) => {
                self.show_open_dialog();
                return Ok(());
            }
            // Save: Ctrl+S
            (true, false, KeyCode::Char('s')) => {
                if let Some(doc) = self.active_document() {
                    if doc.path.is_some() {
                        if let Some(doc) = self.active_document_mut() {
                            let _ = doc.save();
                        }
                    } else {
                        // No path - show save as dialog
                        self.show_save_as_dialog();
                    }
                }
                return Ok(());
            }
            // Save As: Ctrl+Shift+S
            (true, true, KeyCode::Char('S')) => {
                self.show_save_as_dialog();
                return Ok(());
            }
            // Close: Ctrl+W
            (true, false, KeyCode::Char('w')) => {
                self.close_current();
                return Ok(());
            }
            // Next tab: Ctrl+PageDown
            (true, false, KeyCode::PageDown) => {
                self.next_tab();
                return Ok(());
            }
            // Previous tab: Ctrl+PageUp
            (true, false, KeyCode::PageUp) => {
                self.prev_tab();
                return Ok(());
            }
            // Select all: Ctrl+A
            (true, false, KeyCode::Char('a')) => {
                if let Some(doc) = self.active_document_mut() {
                    doc.select_all();
                }
                return Ok(());
            }
            // Cut: Ctrl+X
            (true, false, KeyCode::Char('x')) => {
                let text = self.active_document().map(|doc| doc.selected_text());
                if let Some(text) = text {
                    if !text.is_empty() {
                        let _ = self.clipboard.set_text(&text);
                        if let Some(doc) = self.active_document_mut() {
                            doc.delete_selection();
                        }
                    }
                }
                return Ok(());
            }
            // Copy: Ctrl+C
            (true, false, KeyCode::Char('c')) => {
                if let Some(doc) = self.active_document() {
                    let text = doc.selected_text();
                    if !text.is_empty() {
                        let _ = self.clipboard.set_text(&text);
                    }
                }
                return Ok(());
            }
            // Paste: Ctrl+V
            (true, false, KeyCode::Char('v')) => {
                if let Ok(text) = self.clipboard.get_text() {
                    if let Some(doc) = self.active_document_mut() {
                        doc.insert_str(&text);
                    }
                }
                return Ok(());
            }
            // Find: Ctrl+F
            (true, false, KeyCode::Char('f')) => {
                self.search.open();
                return Ok(());
            }
            // Replace: Ctrl+H
            (true, false, KeyCode::Char('h')) => {
                self.search.open_replace();
                return Ok(());
            }
            // Find Next: F3
            (false, false, KeyCode::F(3)) => {
                if !self.search.query.is_empty() {
                    self.find_next();
                }
                return Ok(());
            }
            // Find Previous: Shift+F3
            (false, true, KeyCode::F(3)) => {
                if !self.search.query.is_empty() {
                    self.find_prev();
                }
                return Ok(());
            }
            _ => {}
        }

        // Handle Alt+number for tab switching
        if alt && !ctrl && !shift {
            if let KeyCode::Char(c @ '0'..='9') = key.code {
                let tab = c.to_digit(10).unwrap() as u8;
                self.go_to_tab(tab);
                return Ok(());
            }
        }

        // Handle regular editor input
        if let Some(doc) = self.active_document_mut() {
            match key.code {
                // Cursor movement
                KeyCode::Left => doc.move_left(shift),
                KeyCode::Right => doc.move_right(shift),
                KeyCode::Up => doc.move_up(shift),
                KeyCode::Down => doc.move_down(shift),
                KeyCode::Home => {
                    if ctrl {
                        doc.move_to_start(shift);
                    } else {
                        doc.move_to_line_start(shift);
                    }
                }
                KeyCode::End => {
                    if ctrl {
                        doc.move_to_end(shift);
                    } else {
                        doc.move_to_line_end(shift);
                    }
                }
                KeyCode::PageUp => doc.page_up(20, shift),
                KeyCode::PageDown => doc.page_down(20, shift),

                // Editing
                KeyCode::Backspace => doc.backspace(),
                KeyCode::Delete => doc.delete(),
                KeyCode::Enter => doc.insert_char('\n'),
                KeyCode::Tab => {
                    // Insert 4 spaces (or tab character based on config)
                    doc.insert_str("    ");
                }
                KeyCode::Insert => doc.toggle_insert_mode(),

                // Character input
                KeyCode::Char(c) => {
                    if !ctrl && !alt {
                        doc.insert_char(c);
                    }
                }

                _ => {}
            }
        }

        Ok(())
    }

    /// Handle keyboard events for the terminal
    fn handle_terminal_key(&mut self, key: event::KeyEvent) -> Result<()> {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        // Don't pass F-keys that are used for focus switching
        match key.code {
            KeyCode::F(2) | KeyCode::F(3) | KeyCode::F(4) => {
                // These are handled by global shortcuts, don't pass to terminal
                return Ok(());
            }
            _ => {}
        }

        // Handle terminal scrolling with Shift+PageUp/PageDown
        if shift {
            match key.code {
                KeyCode::PageUp => {
                    if let Some(ref mut term) = self.terminal {
                        term.scroll_up(10);
                    }
                    return Ok(());
                }
                KeyCode::PageDown => {
                    if let Some(ref mut term) = self.terminal {
                        term.scroll_down(10);
                    }
                    return Ok(());
                }
                KeyCode::Home => {
                    // Scroll to top of scrollback
                    if let Some(ref mut term) = self.terminal {
                        let max = term.max_scrollback();
                        term.scroll_up(max);
                    }
                    return Ok(());
                }
                KeyCode::End => {
                    // Scroll to bottom (current output)
                    if let Some(ref mut term) = self.terminal {
                        term.scroll_to_bottom();
                    }
                    return Ok(());
                }
                _ => {}
            }
        }

        // When user types, scroll to bottom to show current output
        if let Some(ref mut term) = self.terminal {
            if term.is_scrolled_back() {
                // Any regular key input scrolls back to bottom
                match key.code {
                    KeyCode::Char(_) | KeyCode::Enter | KeyCode::Backspace => {
                        term.scroll_to_bottom();
                    }
                    _ => {}
                }
            }
            term.send_key(key.code, key.modifiers)?;
        }
        Ok(())
    }

    /// Handle keyboard events for the file tree
    fn handle_file_tree_key(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.file_tree.move_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.file_tree.move_down();
            }
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                // Enter/Right: expand directory or open file
                if let Some(entry) = self.file_tree.selected_entry() {
                    if entry.is_dir() {
                        self.file_tree.toggle_expand();
                    } else {
                        // Open file in editor
                        let path = entry.path.clone();
                        let _ = self.open_file(path);
                        self.focused_pane = Pane::Editor;
                    }
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                // Left: collapse directory
                if let Some(entry) = self.file_tree.selected_entry() {
                    if entry.is_dir() && entry.expanded {
                        self.file_tree.toggle_expand();
                    }
                }
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.file_tree.go_to_top();
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.file_tree.go_to_bottom();
            }
            KeyCode::PageUp => {
                self.file_tree.page_up(10);
            }
            KeyCode::PageDown => {
                self.file_tree.page_down(10);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle mouse events
    fn handle_mouse_event(&mut self, mouse: event::MouseEvent) -> Result<()> {
        use crate::ui::menu_bar;
        use event::{MouseButton, MouseEventKind};

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Check if clicking on menu bar (row 0)
                if mouse.row == 0 {
                    if let Some(menu_idx) = menu_bar::menu_at_position(self, mouse.column) {
                        if self.menu_open == Some(menu_idx) {
                            // Clicking same menu closes it
                            self.close_menu();
                        } else {
                            // Open this menu
                            self.open_menu(menu_idx);
                        }
                    } else {
                        self.close_menu();
                    }
                    return Ok(());
                }

                // Check if clicking in dropdown menu
                if let Some(menu_idx) = self.menu_open {
                    // Get dropdown bounds
                    let x_pos = self
                        .menu_positions
                        .get(menu_idx)
                        .map(|(start, _, _)| *start)
                        .unwrap_or(0);

                    if let Some((_, items)) = menu_bar::MENUS.get(menu_idx) {
                        let dropdown_height = items.len() as u16 + 2;

                        // Check if click is in dropdown area
                        if mouse.row >= 1 && mouse.row < 1 + dropdown_height {
                            let item_row = (mouse.row - 2) as usize; // -2 for menu bar + border
                            if mouse.row > 1 && item_row < items.len() {
                                if let Some(item_idx) =
                                    menu_bar::item_at_position(menu_idx, item_row)
                                {
                                    self.menu_selected = Some(item_idx);
                                    self.execute_menu_action();
                                    return Ok(());
                                }
                            }
                        }
                    }

                    // Click outside dropdown closes menu
                    self.close_menu();
                    // Don't return - let click be processed normally
                }

                // Check if clicking on a divider to start resize
                if let Some(target) = self.check_divider_click(mouse.column, mouse.row) {
                    self.resizing = Some(target);
                } else if let Some(index) = self.get_file_tree_entry_at(mouse.column, mouse.row) {
                    // Check if clicking in file tree
                    self.focused_pane = Pane::FileTree;
                    self.file_tree.select_index(index);
                } else if let Some((line, col)) =
                    self.get_editor_position_at(mouse.column, mouse.row)
                {
                    // Check if clicking in editor
                    self.focused_pane = Pane::Editor;
                    if let Some(doc) = self.active_document_mut() {
                        doc.move_to(line, col, false);
                    }
                } else {
                    // Click to focus pane
                    self.focus_pane_at(mouse.column, mouse.row);
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.resizing = None;
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if let Some(target) = self.resizing {
                    self.handle_resize_drag(target, mouse.column, mouse.row);
                } else if self.focused_pane == Pane::Editor {
                    // Drag to select text
                    if let Some((line, col)) = self.get_editor_position_at(mouse.column, mouse.row)
                    {
                        if let Some(doc) = self.active_document_mut() {
                            doc.cursor.line = line;
                            doc.cursor.col = col;
                            doc.selection.head = doc.cursor;
                        }
                    }
                }
            }
            MouseEventKind::ScrollUp => match self.focused_pane {
                Pane::FileTree => {
                    self.file_tree.move_up();
                    self.file_tree.move_up();
                    self.file_tree.move_up();
                }
                Pane::Editor => {
                    if let Some(doc) = self.active_document_mut() {
                        doc.scroll_y = doc.scroll_y.saturating_sub(3);
                    }
                }
                Pane::Terminal => {
                    if let Some(ref mut term) = self.terminal {
                        term.scroll_up(3);
                    }
                }
            },
            MouseEventKind::ScrollDown => match self.focused_pane {
                Pane::FileTree => {
                    self.file_tree.move_down();
                    self.file_tree.move_down();
                    self.file_tree.move_down();
                }
                Pane::Editor => {
                    if let Some(doc) = self.active_document_mut() {
                        let max_scroll = doc.line_count().saturating_sub(1);
                        doc.scroll_y = (doc.scroll_y + 3).min(max_scroll);
                    }
                }
                Pane::Terminal => {
                    if let Some(ref mut term) = self.terminal {
                        term.scroll_down(3);
                    }
                }
            },
            _ => {}
        }

        Ok(())
    }

    /// Get the file tree entry index at a screen position
    fn get_file_tree_entry_at(&self, x: u16, y: u16) -> Option<usize> {
        let area = self.file_tree_area?;

        // Check if within the file tree content area
        if x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height {
            let row = (y - area.y) as usize;
            self.file_tree.index_at_row(row)
        } else {
            None
        }
    }

    /// Get the editor document position at a screen position
    fn get_editor_position_at(&self, x: u16, y: u16) -> Option<(usize, usize)> {
        let area = self.editor_area?;

        if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
            return None;
        }

        let doc = self.active_document()?;

        // Calculate gutter width
        let line_count = doc.line_count();
        let digits = if line_count == 0 {
            1
        } else {
            (line_count as f64).log10().floor() as u16 + 1
        };
        let gutter_width = digits.max(4) + 1;

        // Check if click is in gutter
        if x < area.x + gutter_width {
            return None;
        }

        let screen_row = (y - area.y) as usize;
        let screen_col = (x - area.x - gutter_width) as usize;

        let line = doc.scroll_y + screen_row;
        let col = doc.scroll_x + screen_col;

        // Clamp to valid positions
        let line = line.min(doc.line_count().saturating_sub(1));
        let col = col.min(doc.line_len(line));

        Some((line, col))
    }

    /// Cycle focus between visible panes
    fn cycle_focus(&mut self, forward: bool) {
        let panes: Vec<Pane> = {
            let mut p = vec![];
            if self.show_sidebar {
                p.push(Pane::FileTree);
            }
            p.push(Pane::Editor);
            if self.show_terminal {
                p.push(Pane::Terminal);
            }
            p
        };

        if panes.is_empty() {
            return;
        }

        let current_idx = panes
            .iter()
            .position(|&p| p == self.focused_pane)
            .unwrap_or(0);
        let next_idx = if forward {
            (current_idx + 1) % panes.len()
        } else {
            (current_idx + panes.len() - 1) % panes.len()
        };

        self.focused_pane = panes[next_idx];
    }

    /// Check if a click is on a pane divider
    fn check_divider_click(&self, x: u16, y: u16) -> Option<ResizeTarget> {
        if let Ok((cols, rows)) = crossterm::terminal::size() {
            // Calculate positions based on current layout
            // Menu bar is row 0, status bar is last row
            let content_start_y = 1u16;
            let content_end_y = rows.saturating_sub(1);

            if self.show_sidebar {
                let sidebar_width =
                    (cols as f32 * self.sidebar_width_percent as f32 / 100.0) as u16;
                let sidebar_width = sidebar_width.clamp(15, cols.saturating_sub(40));

                // Check if clicking on vertical divider (sidebar border)
                // Allow a few pixels tolerance
                if x >= sidebar_width.saturating_sub(1)
                    && x <= sidebar_width + 1
                    && y >= content_start_y
                    && y < content_end_y
                {
                    return Some(ResizeTarget::Sidebar);
                }
            }

            if self.show_terminal {
                let content_height = content_end_y.saturating_sub(content_start_y);
                let terminal_height =
                    (content_height as f32 * self.terminal_height_percent as f32 / 100.0) as u16;
                let terminal_height = terminal_height.clamp(5, content_height.saturating_sub(10));

                // Terminal divider Y position (the line with "Terminal" title)
                let divider_y = content_end_y.saturating_sub(terminal_height);

                // Adjust X position for sidebar
                let editor_start_x = if self.show_sidebar {
                    let sidebar_width =
                        (cols as f32 * self.sidebar_width_percent as f32 / 100.0) as u16;
                    sidebar_width.clamp(15, cols.saturating_sub(40))
                } else {
                    0
                };

                // Check if clicking on horizontal divider (terminal border)
                if y >= divider_y.saturating_sub(1) && y <= divider_y + 1 && x >= editor_start_x {
                    return Some(ResizeTarget::Terminal);
                }
            }
        }
        None
    }

    /// Focus the pane at the given screen coordinates
    fn focus_pane_at(&mut self, x: u16, y: u16) {
        if let Ok((cols, rows)) = crossterm::terminal::size() {
            // Skip menu bar (row 0) and status bar (last row)
            if y == 0 || y >= rows.saturating_sub(1) {
                return;
            }

            let content_start_y = 1u16;
            let content_end_y = rows.saturating_sub(1);
            let content_height = content_end_y.saturating_sub(content_start_y);

            // Check sidebar first
            if self.show_sidebar {
                let sidebar_width =
                    (cols as f32 * self.sidebar_width_percent as f32 / 100.0) as u16;
                let sidebar_width = sidebar_width.clamp(15, cols.saturating_sub(40));

                if x < sidebar_width {
                    self.focused_pane = Pane::FileTree;
                    return;
                }
            }

            // Check editor vs terminal
            if self.show_terminal {
                let terminal_height =
                    (content_height as f32 * self.terminal_height_percent as f32 / 100.0) as u16;
                let terminal_height = terminal_height.clamp(5, content_height.saturating_sub(10));
                let divider_y = content_end_y.saturating_sub(terminal_height);

                if y >= divider_y {
                    self.focused_pane = Pane::Terminal;
                } else {
                    self.focused_pane = Pane::Editor;
                }
            } else {
                self.focused_pane = Pane::Editor;
            }
        }
    }

    /// Handle dragging to resize a pane
    fn handle_resize_drag(&mut self, target: ResizeTarget, x: u16, _y: u16) {
        // Get terminal size for percentage calculations
        if let Ok((cols, rows)) = crossterm::terminal::size() {
            match target {
                ResizeTarget::Sidebar => {
                    // Calculate new sidebar width percentage
                    let new_percent = ((x as f32 / cols as f32) * 100.0) as u16;
                    self.sidebar_width_percent = new_percent.clamp(10, 50);
                }
                ResizeTarget::Terminal => {
                    // Calculate new terminal height percentage
                    let new_percent = ((1.0 - (_y as f32 / rows as f32)) * 100.0) as u16;
                    self.terminal_height_percent = new_percent.clamp(10, 70);
                }
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
