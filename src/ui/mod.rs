pub mod dialog;
mod editor;
mod file_tree;
mod layout;
pub mod menu_bar;
mod status_bar;
mod tab_bar;
mod terminal;

use crate::app::App;
use ratatui::prelude::*;

pub use layout::Pane;
pub use menu_bar::MenuAction;

/// Draw the entire UI
pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Check minimum size
    if area.width < 80 || area.height < 24 {
        draw_size_warning(frame, area);
        return;
    }

    // Main layout: menu bar, content, status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Menu bar
            Constraint::Min(10),   // Content area
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    // Draw menu bar (just the bar, not dropdown yet)
    menu_bar::draw_bar(frame, app, main_chunks[0]);

    // Draw main content (sidebar + editor/terminal)
    layout::draw_content(frame, app, main_chunks[1]);

    // Draw status bar
    status_bar::draw(frame, app, main_chunks[2]);

    // Draw menu dropdown if open
    if app.menu_open.is_some() {
        menu_bar::draw_dropdown(frame, app, main_chunks[0]);
    }

    // Draw dialog LAST so it appears on top of everything
    if app.dialog.is_some() {
        dialog::draw_dialog(frame, app);
    }
}

/// Draw a warning when terminal is too small
fn draw_size_warning(frame: &mut Frame, area: Rect) {
    use ratatui::widgets::{Block, Borders, Paragraph};

    let warning = Paragraph::new("Terminal too small!\nMinimum: 80x24")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("gterm"));

    frame.render_widget(warning, area);
}
