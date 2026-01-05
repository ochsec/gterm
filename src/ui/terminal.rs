use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Tabs},
};

/// Draw the terminal pane with tab bar for multiple terminals
pub fn draw(frame: &mut Frame, app: &mut App, area: Rect, focused: bool) {
    let border_color = if focused {
        app.theme.border_focused
    } else {
        app.theme.border
    };

    // Split area: tab bar on top, terminal content below
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab bar
            Constraint::Min(1),    // Terminal content
        ])
        .split(area);

    // Draw terminal tab bar
    draw_terminal_tabs(frame, app, chunks[0], focused);

    // Store terminal area for resize detection
    app.terminal_area = Some(area);

    // Draw active terminal content
    draw_terminal_content(frame, app, chunks[1], focused);
}

/// Draw the terminal tab bar
fn draw_terminal_tabs(frame: &mut Frame, app: &App, area: Rect, focused: bool) {
    if app.terminals.is_empty() {
        // No terminals - show empty bar
        let block = Block::default().style(Style::default().bg(app.theme.statusbar_bg));
        frame.render_widget(block, area);
        return;
    }

    // Build tab titles
    let titles: Vec<Line> = app
        .terminals
        .iter()
        .enumerate()
        .map(|(i, term)| {
            let parser = term.screen();
            let screen = parser.screen();
            let scroll_offset = screen.scrollback();

            let title = if scroll_offset > 0 {
                format!(" Terminal {} [-{}] ", i + 1, scroll_offset)
            } else {
                format!(" Terminal {} ", i + 1)
            };
            Line::from(title)
        })
        .collect();

    let tabs = Tabs::new(titles)
        .select(app.active_terminal)
        .style(Style::default().fg(app.theme.fg).bg(app.theme.terminal_bg))
        .highlight_style(
            Style::default()
                .fg(app.theme.fg)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .divider("|");

    frame.render_widget(tabs, area);
}

/// Draw the terminal content for the active terminal
fn draw_terminal_content(frame: &mut Frame, app: &mut App, area: Rect, focused: bool) {
    let block = Block::default()
        .borders(Borders::NONE)
        .style(Style::default().bg(app.theme.terminal_bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Render terminal content from PTY
    if let Some(term) = app.active_terminal() {
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
        // No terminal - show placeholder with hint
        let placeholder = Paragraph::new("No terminal. Press Ctrl+Shift+N to create one.")
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
