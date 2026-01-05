use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Menu item definition
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: &'static str,
    pub shortcut: Option<&'static str>,
    pub action: MenuAction,
    pub enabled: bool,
}

/// Menu actions that can be triggered
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    // File menu
    NewFile,
    OpenFile,
    Save,
    SaveAs,
    SaveAll,
    Close,
    CloseAll,
    Quit,

    // Edit menu
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,

    // Search menu
    Find,
    FindNext,
    FindPrevious,
    Replace,
    GoToLine,

    // View menu
    ToggleSidebar,
    ToggleEditor,
    ToggleTerminal,
    FocusEditor,
    FocusFileTree,
    FocusTerminal,

    // Terminal menu
    NewTerminal,
    CloseTerminal,
    NextTerminal,
    PrevTerminal,

    // Help menu
    About,

    // Separator (not a real action)
    Separator,
}

/// The available menus
pub const MENUS: &[(&str, &[MenuItem])] = &[
    (
        "File",
        &[
            MenuItem {
                label: "New",
                shortcut: Some("Ctrl+N"),
                action: MenuAction::NewFile,
                enabled: true,
            },
            MenuItem {
                label: "Open...",
                shortcut: Some("Ctrl+O"),
                action: MenuAction::OpenFile,
                enabled: true,
            },
            MenuItem {
                label: "─────────",
                shortcut: None,
                action: MenuAction::Separator,
                enabled: false,
            },
            MenuItem {
                label: "Save",
                shortcut: Some("Ctrl+S"),
                action: MenuAction::Save,
                enabled: true,
            },
            MenuItem {
                label: "Save As...",
                shortcut: Some("Ctrl+Shift+S"),
                action: MenuAction::SaveAs,
                enabled: true,
            },
            MenuItem {
                label: "Save All",
                shortcut: None,
                action: MenuAction::SaveAll,
                enabled: true,
            },
            MenuItem {
                label: "─────────",
                shortcut: None,
                action: MenuAction::Separator,
                enabled: false,
            },
            MenuItem {
                label: "Close",
                shortcut: Some("Ctrl+W"),
                action: MenuAction::Close,
                enabled: true,
            },
            MenuItem {
                label: "Close All",
                shortcut: None,
                action: MenuAction::CloseAll,
                enabled: true,
            },
            MenuItem {
                label: "─────────",
                shortcut: None,
                action: MenuAction::Separator,
                enabled: false,
            },
            MenuItem {
                label: "Quit",
                shortcut: Some("Ctrl+Q"),
                action: MenuAction::Quit,
                enabled: true,
            },
        ],
    ),
    (
        "Edit",
        &[
            MenuItem {
                label: "Undo",
                shortcut: Some("Ctrl+Z"),
                action: MenuAction::Undo,
                enabled: true,
            },
            MenuItem {
                label: "Redo",
                shortcut: Some("Ctrl+Y"),
                action: MenuAction::Redo,
                enabled: true,
            },
            MenuItem {
                label: "─────────",
                shortcut: None,
                action: MenuAction::Separator,
                enabled: false,
            },
            MenuItem {
                label: "Cut",
                shortcut: Some("Ctrl+X"),
                action: MenuAction::Cut,
                enabled: true,
            },
            MenuItem {
                label: "Copy",
                shortcut: Some("Ctrl+C"),
                action: MenuAction::Copy,
                enabled: true,
            },
            MenuItem {
                label: "Paste",
                shortcut: Some("Ctrl+V"),
                action: MenuAction::Paste,
                enabled: true,
            },
            MenuItem {
                label: "─────────",
                shortcut: None,
                action: MenuAction::Separator,
                enabled: false,
            },
            MenuItem {
                label: "Select All",
                shortcut: Some("Ctrl+A"),
                action: MenuAction::SelectAll,
                enabled: true,
            },
        ],
    ),
    (
        "Search",
        &[
            MenuItem {
                label: "Find...",
                shortcut: Some("Ctrl+F"),
                action: MenuAction::Find,
                enabled: true,
            },
            MenuItem {
                label: "Find Next",
                shortcut: Some("F3"),
                action: MenuAction::FindNext,
                enabled: true,
            },
            MenuItem {
                label: "Find Previous",
                shortcut: Some("Shift+F3"),
                action: MenuAction::FindPrevious,
                enabled: true,
            },
            MenuItem {
                label: "Replace...",
                shortcut: Some("Ctrl+H"),
                action: MenuAction::Replace,
                enabled: true,
            },
            MenuItem {
                label: "─────────",
                shortcut: None,
                action: MenuAction::Separator,
                enabled: false,
            },
            MenuItem {
                label: "Go to Line...",
                shortcut: Some("Ctrl+L"),
                action: MenuAction::GoToLine,
                enabled: true,
            },
        ],
    ),
    (
        "View",
        &[
            MenuItem {
                label: "Toggle Sidebar",
                shortcut: Some("Ctrl+B"),
                action: MenuAction::ToggleSidebar,
                enabled: true,
            },
            MenuItem {
                label: "Toggle Editor",
                shortcut: Some("Ctrl+E"),
                action: MenuAction::ToggleEditor,
                enabled: true,
            },
            MenuItem {
                label: "Toggle Terminal",
                shortcut: Some("Ctrl+T"),
                action: MenuAction::ToggleTerminal,
                enabled: true,
            },
            MenuItem {
                label: "─────────",
                shortcut: None,
                action: MenuAction::Separator,
                enabled: false,
            },
            MenuItem {
                label: "Focus File Tree",
                shortcut: Some("F3"),
                action: MenuAction::FocusFileTree,
                enabled: true,
            },
            MenuItem {
                label: "Focus Terminal",
                shortcut: Some("F4"),
                action: MenuAction::FocusTerminal,
                enabled: true,
            },
        ],
    ),
    (
        "Terminal",
        &[
            MenuItem {
                label: "New Terminal",
                shortcut: Some("Ctrl+N"),
                action: MenuAction::NewTerminal,
                enabled: true,
            },
            MenuItem {
                label: "Close Terminal",
                shortcut: Some("Ctrl+W"),
                action: MenuAction::CloseTerminal,
                enabled: true,
            },
            MenuItem {
                label: "─────────",
                shortcut: None,
                action: MenuAction::Separator,
                enabled: false,
            },
            MenuItem {
                label: "Next Terminal",
                shortcut: Some("Alt+."),
                action: MenuAction::NextTerminal,
                enabled: true,
            },
            MenuItem {
                label: "Previous Terminal",
                shortcut: Some("Alt+,"),
                action: MenuAction::PrevTerminal,
                enabled: true,
            },
        ],
    ),
    (
        "Help",
        &[MenuItem {
            label: "About gterm",
            shortcut: None,
            action: MenuAction::About,
            enabled: true,
        }],
    ),
];

/// Draw just the menu bar (not the dropdown)
pub fn draw_bar(frame: &mut Frame, app: &mut App, area: Rect) {
    let bg_style = Style::default()
        .fg(app.theme.menubar_fg)
        .bg(app.theme.menubar_bg);

    let selected_style = Style::default()
        .fg(app.theme.menubar_bg)
        .bg(app.theme.menubar_fg);

    let mut spans: Vec<Span> = Vec::new();
    let mut x_offset = 0u16;

    // Store menu positions for click detection
    app.menu_positions.clear();

    for (i, (name, _items)) in MENUS.iter().enumerate() {
        let label = format!(" {} ", name);
        let width = label.len() as u16;

        let style = if app.menu_open == Some(i) {
            selected_style
        } else {
            bg_style
        };

        spans.push(Span::styled(label, style));

        // Store position for this menu
        app.menu_positions.push((x_offset, x_offset + width, i));
        x_offset += width;
    }

    // Fill remaining space
    let title = "gterm";
    let remaining = area.width.saturating_sub(x_offset + title.len() as u16 + 1);
    spans.push(Span::styled(" ".repeat(remaining as usize), bg_style));
    spans.push(Span::styled(title, bg_style));
    spans.push(Span::styled(" ", bg_style));

    let line = Line::from(spans);
    let menu_bar = Paragraph::new(line);
    frame.render_widget(menu_bar, area);
}

/// Draw the dropdown menu (call this AFTER drawing all other content)
pub fn draw_dropdown(frame: &mut Frame, app: &App, menu_bar_area: Rect) {
    let Some(menu_idx) = app.menu_open else {
        return;
    };

    let Some((_, items)) = MENUS.get(menu_idx) else {
        return;
    };

    // Calculate dropdown position
    let x_pos = app
        .menu_positions
        .get(menu_idx)
        .map(|(start, _, _)| *start)
        .unwrap_or(0);

    // Calculate dropdown dimensions
    let max_label_width = items
        .iter()
        .map(|item| item.label.len())
        .max()
        .unwrap_or(10);
    let max_shortcut_width = items
        .iter()
        .filter_map(|item| item.shortcut)
        .map(|s| s.len())
        .max()
        .unwrap_or(0);

    let dropdown_width = (max_label_width + max_shortcut_width + 6) as u16; // padding + borders
    let dropdown_height = items.len() as u16 + 2; // +2 for borders

    // Get total screen size
    let screen_width = frame.area().width;

    let dropdown_area = Rect {
        x: x_pos.min(screen_width.saturating_sub(dropdown_width)),
        y: menu_bar_area.y + 1,
        width: dropdown_width,
        height: dropdown_height,
    };

    // Clear the area behind the dropdown
    frame.render_widget(Clear, dropdown_area);

    // Draw dropdown background/border
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.border))
        .style(Style::default().bg(app.theme.menubar_bg));

    let inner = block.inner(dropdown_area);
    frame.render_widget(block, dropdown_area);

    // Draw menu items
    let normal_style = Style::default()
        .fg(app.theme.menubar_fg)
        .bg(app.theme.menubar_bg);
    let disabled_style = Style::default()
        .fg(app.theme.line_number)
        .bg(app.theme.menubar_bg);
    let selected_style = Style::default()
        .fg(app.theme.menubar_bg)
        .bg(app.theme.statusbar_bg);

    let mut lines: Vec<Line> = Vec::new();
    let inner_width = inner.width as usize;

    for (i, item) in items.iter().enumerate() {
        let is_selected = app.menu_selected == Some(i);

        let style = if item.action == MenuAction::Separator {
            disabled_style
        } else if !item.enabled {
            disabled_style
        } else if is_selected {
            selected_style
        } else {
            normal_style
        };

        let shortcut_str = item.shortcut.unwrap_or("");
        let label_len = item.label.len();
        let shortcut_len = shortcut_str.len();
        let padding = inner_width.saturating_sub(label_len + shortcut_len + 2);

        let line_text = format!(
            " {}{:padding$}{}",
            item.label,
            "",
            shortcut_str,
            padding = padding
        );

        // Ensure line fills the width
        let padded_line = format!("{:<width$}", line_text, width = inner_width);

        lines.push(Line::from(Span::styled(padded_line, style)));
    }

    let content = Paragraph::new(lines);
    frame.render_widget(content, inner);
}

/// Get the menu index at a given x position in the menu bar
pub fn menu_at_position(app: &App, x: u16) -> Option<usize> {
    for (start, end, idx) in &app.menu_positions {
        if x >= *start && x < *end {
            return Some(*idx);
        }
    }
    None
}

/// Get the menu item at a given y position in the dropdown (relative to dropdown top)
pub fn item_at_position(menu_idx: usize, y: usize) -> Option<usize> {
    if let Some((_, items)) = MENUS.get(menu_idx) {
        if y < items.len() {
            // Skip separators
            if items[y].action != MenuAction::Separator && items[y].enabled {
                return Some(y);
            }
        }
    }
    None
}
