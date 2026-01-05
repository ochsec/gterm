use crate::app::App;
use ratatui::prelude::*;

use super::{editor, file_tree, search_bar, tab_bar, terminal};

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
    let search_height = search_bar::height(app);

    match (app.show_editor, app.show_terminal) {
        // Both editor and terminal visible
        (true, true) => {
            let terminal_height =
                (area.height as f32 * app.terminal_height_percent as f32 / 100.0) as u16;
            let terminal_height = terminal_height.clamp(5, area.height.saturating_sub(10));

            let v_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),               // Tab bar
                    Constraint::Min(5),                  // Editor
                    Constraint::Length(search_height),   // Search bar (0 or 1)
                    Constraint::Length(terminal_height), // Terminal
                ])
                .split(area);

            tab_bar::draw(frame, app, v_chunks[0]);
            editor::draw(frame, app, v_chunks[1], app.focused_pane == Pane::Editor);
            if search_height > 0 {
                search_bar::draw(frame, app, v_chunks[2]);
            }
            terminal::draw(frame, app, v_chunks[3], app.focused_pane == Pane::Terminal);
        }
        // Only editor visible
        (true, false) => {
            let v_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),             // Tab bar
                    Constraint::Min(5),                // Editor
                    Constraint::Length(search_height), // Search bar (0 or 1)
                ])
                .split(area);

            tab_bar::draw(frame, app, v_chunks[0]);
            editor::draw(frame, app, v_chunks[1], app.focused_pane == Pane::Editor);
            if search_height > 0 {
                search_bar::draw(frame, app, v_chunks[2]);
            }
        }
        // Only terminal visible
        (false, true) => {
            terminal::draw(frame, app, area, app.focused_pane == Pane::Terminal);
        }
        // Neither visible - show empty area or default to terminal
        (false, false) => {
            // Show at least terminal if both are hidden
            terminal::draw(frame, app, area, app.focused_pane == Pane::Terminal);
        }
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
