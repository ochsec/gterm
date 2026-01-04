use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Draw the terminal pane
pub fn draw(frame: &mut Frame, app: &App, area: Rect, focused: bool) {
    let border_color = if focused {
        app.theme.border_focused
    } else {
        app.theme.border
    };

    let title = if focused { " Terminal " } else { " Terminal " };

    let block = Block::default()
        .title(title)
        .borders(Borders::TOP)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(app.theme.terminal_bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Placeholder terminal content
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let shell_name = std::path::Path::new(&shell)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("sh");

    let prompt_style = Style::default()
        .fg(Color::Rgb(100, 200, 100))
        .bg(app.theme.terminal_bg);

    let text_style = Style::default().fg(app.theme.fg).bg(app.theme.terminal_bg);

    let cursor_style = Style::default().fg(app.theme.terminal_bg).bg(app.theme.fg);

    let lines = vec![
        Line::from(vec![
            Span::styled(format!("{}$ ", shell_name), prompt_style),
            Span::styled("", text_style),
        ]),
        Line::from(vec![
            Span::styled(format!("{}$ ", shell_name), prompt_style),
            Span::styled("echo \"Welcome to gterm!\"", text_style),
        ]),
        Line::from(vec![Span::styled("Welcome to gterm!", text_style)]),
        Line::from(vec![
            Span::styled(format!("{}$ ", shell_name), prompt_style),
            Span::styled(" ", cursor_style),
        ]),
    ];

    let content = Paragraph::new(lines);
    frame.render_widget(content, inner);
}
