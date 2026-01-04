use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Draw the terminal pane
pub fn draw(frame: &mut Frame, app: &mut App, area: Rect, focused: bool) {
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

    // Store terminal area for resize detection
    app.terminal_area = Some(area);

    // Render terminal content from PTY
    if let Some(ref term) = app.terminal {
        let parser = term.screen();
        let screen = parser.screen();

        let mut lines: Vec<Line> = Vec::new();

        for row in 0..inner.height {
            let mut spans: Vec<Span> = Vec::new();

            for col in 0..inner.width {
                let cell = screen.cell(row, col);

                if let Some(cell) = cell {
                    let c = cell.contents();
                    let display_char = if c.is_empty() { " ".to_string() } else { c };

                    // Convert vt100 colors to ratatui colors
                    let fg = convert_color(cell.fgcolor(), app.theme.fg);
                    let bg = convert_color(cell.bgcolor(), app.theme.terminal_bg);

                    let mut style = Style::default().fg(fg).bg(bg);

                    if cell.bold() {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    if cell.italic() {
                        style = style.add_modifier(Modifier::ITALIC);
                    }
                    if cell.underline() {
                        style = style.add_modifier(Modifier::UNDERLINED);
                    }
                    if cell.inverse() {
                        style = style.fg(bg).bg(fg);
                    }

                    spans.push(Span::styled(display_char.to_string(), style));
                } else {
                    spans.push(Span::styled(
                        " ",
                        Style::default().bg(app.theme.terminal_bg),
                    ));
                }
            }

            lines.push(Line::from(spans));
        }

        let content = Paragraph::new(lines);
        frame.render_widget(content, inner);

        // Draw cursor if terminal is focused
        if focused {
            let cursor_pos = screen.cursor_position();
            let cursor_x = inner.x + cursor_pos.1;
            let cursor_y = inner.y + cursor_pos.0;

            if cursor_x < inner.x + inner.width && cursor_y < inner.y + inner.height {
                // Get the character under the cursor
                let cursor_cell = screen.cell(cursor_pos.0, cursor_pos.1);
                let cursor_char = cursor_cell
                    .map(|c| {
                        let contents = c.contents();
                        if contents.is_empty() {
                            " ".to_string()
                        } else {
                            contents
                        }
                    })
                    .unwrap_or_else(|| " ".to_string());

                let cursor_style = Style::default().fg(app.theme.terminal_bg).bg(app.theme.fg);

                frame.render_widget(
                    Paragraph::new(cursor_char).style(cursor_style),
                    Rect::new(cursor_x, cursor_y, 1, 1),
                );
            }
        }
    } else {
        // No terminal - show placeholder
        let placeholder = Paragraph::new("Terminal not available")
            .style(Style::default().fg(app.theme.line_number))
            .alignment(Alignment::Center);
        frame.render_widget(placeholder, inner);
    }
}

/// Convert vt100 color to ratatui Color
fn convert_color(color: vt100::Color, default: Color) -> Color {
    match color {
        vt100::Color::Default => default,
        vt100::Color::Idx(idx) => {
            // Standard 16-color palette
            match idx {
                0 => Color::Black,
                1 => Color::Red,
                2 => Color::Green,
                3 => Color::Yellow,
                4 => Color::Blue,
                5 => Color::Magenta,
                6 => Color::Cyan,
                7 => Color::White,
                8 => Color::DarkGray,
                9 => Color::LightRed,
                10 => Color::LightGreen,
                11 => Color::LightYellow,
                12 => Color::LightBlue,
                13 => Color::LightMagenta,
                14 => Color::LightCyan,
                15 => Color::Gray,
                // 256-color palette
                16..=231 => {
                    // 6x6x6 color cube
                    let idx = idx - 16;
                    let r = (idx / 36) * 51;
                    let g = ((idx / 6) % 6) * 51;
                    let b = (idx % 6) * 51;
                    Color::Rgb(r, g, b)
                }
                232..=255 => {
                    // Grayscale
                    let gray = (idx - 232) * 10 + 8;
                    Color::Rgb(gray, gray, gray)
                }
                _ => default,
            }
        }
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}
