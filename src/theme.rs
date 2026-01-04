use ratatui::style::Color;

/// Color theme for the application
#[derive(Debug, Clone)]
pub struct Theme {
    /// Background color for the main UI
    pub bg: Color,
    /// Foreground (text) color
    pub fg: Color,
    /// Background for the sidebar/file tree
    pub sidebar_bg: Color,
    /// Background for the editor
    pub editor_bg: Color,
    /// Background for the terminal
    pub terminal_bg: Color,
    /// Background for the status bar
    pub statusbar_bg: Color,
    /// Foreground for the status bar
    pub statusbar_fg: Color,
    /// Background for the menu bar
    pub menubar_bg: Color,
    /// Foreground for the menu bar
    pub menubar_fg: Color,
    /// Background for the tab bar
    pub tabbar_bg: Color,
    /// Active tab background
    pub tab_active_bg: Color,
    /// Active tab foreground
    pub tab_active_fg: Color,
    /// Inactive tab background
    pub tab_inactive_bg: Color,
    /// Inactive tab foreground
    pub tab_inactive_fg: Color,
    /// Border color
    pub border: Color,
    /// Border color for focused pane
    pub border_focused: Color,
    /// Line number color
    pub line_number: Color,
    /// Current line number color
    pub line_number_current: Color,
    /// Current line highlight
    pub line_highlight: Color,
    /// Selection background
    pub selection_bg: Color,
    /// Cursor color
    pub cursor: Color,
    /// Directory color in file tree
    pub tree_dir: Color,
    /// File color in file tree
    pub tree_file: Color,
    /// Selected item in file tree
    pub tree_selected_bg: Color,
}

impl Theme {
    /// Create a dark theme inspired by Geany's dark mode
    pub fn dark() -> Self {
        Self {
            bg: Color::Rgb(30, 30, 30),
            fg: Color::Rgb(212, 212, 212),
            sidebar_bg: Color::Rgb(37, 37, 38),
            editor_bg: Color::Rgb(30, 30, 30),
            terminal_bg: Color::Rgb(24, 24, 24),
            statusbar_bg: Color::Rgb(0, 122, 204),
            statusbar_fg: Color::Rgb(255, 255, 255),
            menubar_bg: Color::Rgb(60, 60, 60),
            menubar_fg: Color::Rgb(212, 212, 212),
            tabbar_bg: Color::Rgb(45, 45, 45),
            tab_active_bg: Color::Rgb(30, 30, 30),
            tab_active_fg: Color::Rgb(255, 255, 255),
            tab_inactive_bg: Color::Rgb(45, 45, 45),
            tab_inactive_fg: Color::Rgb(150, 150, 150),
            border: Color::Rgb(60, 60, 60),
            border_focused: Color::Rgb(0, 122, 204),
            line_number: Color::Rgb(133, 133, 133),
            line_number_current: Color::Rgb(200, 200, 200),
            line_highlight: Color::Rgb(40, 40, 40),
            selection_bg: Color::Rgb(38, 79, 120),
            cursor: Color::Rgb(255, 255, 255),
            tree_dir: Color::Rgb(220, 220, 170),
            tree_file: Color::Rgb(212, 212, 212),
            tree_selected_bg: Color::Rgb(62, 62, 62),
        }
    }
}
