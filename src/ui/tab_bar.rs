use crate::app::App;
use ratatui::{prelude::*, widgets::Paragraph};

/// Draw the tab bar showing open documents
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let bg_style = Style::default()
        .fg(app.theme.tab_inactive_fg)
        .bg(app.theme.tabbar_bg);

    let active_style = Style::default()
        .fg(app.theme.tab_active_fg)
        .bg(app.theme.tab_active_bg);

    let inactive_style = Style::default()
        .fg(app.theme.tab_inactive_fg)
        .bg(app.theme.tab_inactive_bg);

    let modified_style = Style::default()
        .fg(Color::Rgb(255, 200, 100))
        .bg(app.theme.tab_active_bg);

    let mut spans: Vec<Span> = Vec::new();
    let mut total_width = 0u16;

    for (i, doc) in app.documents.iter().enumerate() {
        let is_active = i == app.active_doc;
        let title = doc.title();
        let modified_marker = if doc.modified { "*" } else { "" };

        // Format: " title* │" or " title │"
        let tab_text = format!(" {}{} ", title, modified_marker);
        let tab_width = tab_text.len() as u16 + 1; // +1 for separator

        if total_width + tab_width > area.width {
            // Not enough room for more tabs
            break;
        }

        let style = if is_active {
            active_style
        } else {
            inactive_style
        };

        spans.push(Span::styled(tab_text, style));

        // Add separator
        spans.push(Span::styled("│", bg_style));

        total_width += tab_width;
    }

    // Fill the rest with background
    let remaining = area.width.saturating_sub(total_width);
    if remaining > 0 {
        spans.push(Span::styled(" ".repeat(remaining as usize), bg_style));
    }

    let line = Line::from(spans);
    let tab_bar = Paragraph::new(line);

    frame.render_widget(tab_bar, area);
}
