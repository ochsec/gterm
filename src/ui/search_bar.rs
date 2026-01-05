use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Draw the search bar at the bottom of the editor area
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    if !app.search.active {
        return;
    }

    // Search bar is 1 line at the bottom of the editor area
    let search_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    };

    let bg_style = Style::default()
        .bg(app.theme.statusbar_bg)
        .fg(app.theme.statusbar_fg);

    // Build the search bar content
    let label = if app.search.replace_mode {
        "Replace: "
    } else {
        "Find: "
    };

    let query = &app.search.query;
    let match_info = app.search.match_info();

    // Create spans for the search bar
    let spans = vec![
        Span::styled(label, bg_style.add_modifier(Modifier::BOLD)),
        Span::styled(query.as_str(), bg_style),
        Span::styled(
            "│",
            Style::default().fg(app.theme.fg).bg(app.theme.statusbar_bg),
        ), // Cursor
        Span::styled(
            format!(" {}", match_info),
            Style::default()
                .fg(app.theme.line_number)
                .bg(app.theme.statusbar_bg),
        ),
    ];

    // Calculate padding to fill the width
    let content_width = label.len() + query.len() + 1 + match_info.len() + 1;
    let padding = area.width.saturating_sub(content_width as u16) as usize;

    let mut all_spans = spans;
    all_spans.push(Span::styled(
        " ".repeat(padding),
        Style::default().bg(app.theme.statusbar_bg),
    ));

    // Add hint for shortcuts
    let hint = " [Enter: next, Shift+F3: prev, Esc: close]";
    let hint_style = Style::default()
        .fg(app.theme.line_number)
        .bg(app.theme.statusbar_bg);

    // Check if there's room for the hint
    if area.width as usize > content_width + hint.len() {
        all_spans.push(Span::styled(hint, hint_style));
    }

    let line = Line::from(all_spans);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, search_area);
}

/// Calculate the height of the search bar (0 if not active, 1 if active)
pub fn height(app: &App) -> u16 {
    if app.search.active {
        1
    } else {
        0
    }
}
