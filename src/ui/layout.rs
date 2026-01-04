use crate::app::App;
use ratatui::prelude::*;

use super::{editor, file_tree, tab_bar, terminal};

/// Represents which pane is currently focused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    FileTree,
    Editor,
    Terminal,
}

/// Stores the calculated areas for hit testing
#[derive(Debug, Clone, Default)]
pub struct LayoutAreas {
    pub file_tree: Option<Rect>,
    pub editor: Option<Rect>,
    pub terminal: Option<Rect>,
    pub sidebar_divider: Option<Rect>,
    pub terminal_divider: Option<Rect>,
}

/// Draw the main content area (file tree + editor + terminal)
pub fn draw_content(frame: &mut Frame, app: &mut App, area: Rect) {
    if app.show_sidebar {
        // Split horizontally: sidebar | main content
        let sidebar_width = (area.width as f32 * app.sidebar_width_percent as f32 / 100.0) as u16;
        let sidebar_width = sidebar_width.clamp(15, area.width.saturating_sub(40));

        let h_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(sidebar_width), Constraint::Min(40)])
            .split(area);

        // Draw file tree in left panel
        let focused = app.focused_pane == Pane::FileTree;
        file_tree::draw(frame, app, h_chunks[0], focused);

        // Draw editor + terminal in right panel
        draw_editor_terminal(frame, app, h_chunks[1]);
    } else {
        // No sidebar, just editor + terminal
        draw_editor_terminal(frame, app, area);
    }
}

/// Draw the editor and terminal panes (stacked vertically)
fn draw_editor_terminal(frame: &mut Frame, app: &mut App, area: Rect) {
    if app.show_terminal {
        // Split vertically: editor on top, terminal on bottom
        let terminal_height =
            (area.height as f32 * app.terminal_height_percent as f32 / 100.0) as u16;
        let terminal_height = terminal_height.clamp(5, area.height.saturating_sub(10));

        let v_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),               // Tab bar
                Constraint::Min(5),                  // Editor
                Constraint::Length(terminal_height), // Terminal
            ])
            .split(area);

        // Draw tab bar
        tab_bar::draw(frame, app, v_chunks[0]);

        // Draw editor
        let editor_focused = app.focused_pane == Pane::Editor;
        editor::draw(frame, app, v_chunks[1], editor_focused);

        // Draw terminal
        terminal::draw(frame, app, v_chunks[2], app.focused_pane == Pane::Terminal);
    } else {
        // No terminal, just tab bar + editor
        let v_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Tab bar
                Constraint::Min(5),    // Editor
            ])
            .split(area);

        // Draw tab bar
        tab_bar::draw(frame, app, v_chunks[0]);

        // Draw editor
        let editor_focused = app.focused_pane == Pane::Editor;
        editor::draw(frame, app, v_chunks[1], editor_focused);
    }
}

/// Determine which pane contains the given coordinates
pub fn pane_at_position(app: &App, x: u16, y: u16, total_area: Rect) -> Option<Pane> {
    // Skip menu bar (row 0) and status bar (last row)
    if y == 0 || y >= total_area.height.saturating_sub(1) {
        return None;
    }

    let content_y = y - 1; // Adjust for menu bar
    let content_height = total_area.height.saturating_sub(2);

    if app.show_sidebar {
        let sidebar_width =
            (total_area.width as f32 * app.sidebar_width_percent as f32 / 100.0) as u16;
        let sidebar_width = sidebar_width.clamp(15, total_area.width.saturating_sub(40));

        if x < sidebar_width {
            return Some(Pane::FileTree);
        }
    }

    // In the editor/terminal area
    if app.show_terminal {
        let terminal_height =
            (content_height as f32 * app.terminal_height_percent as f32 / 100.0) as u16;
        let terminal_height = terminal_height.clamp(5, content_height.saturating_sub(10));
        let editor_height = content_height - terminal_height - 1; // -1 for tab bar

        if content_y <= editor_height {
            return Some(Pane::Editor);
        } else {
            return Some(Pane::Terminal);
        }
    }

    Some(Pane::Editor)
}
