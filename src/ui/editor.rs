use crate::app::App;
use crate::theme::Theme;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Draw the code editor pane
pub fn draw(frame: &mut Frame, app: &mut App, area: Rect, focused: bool) {
    let border_color = if focused {
        app.theme.border_focused
    } else {
        app.theme.border
    };

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(app.theme.editor_bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Store editor area for mouse handling
    app.editor_area = Some(inner);

    // Copy theme colors we need
    let theme = app.theme.clone();

    // Get visible area dimensions
    let visible_lines = inner.height as usize;

    // Get document info we need
    let doc_info = if let Some(doc) = app.active_document_mut() {
        let line_count = doc.line_count();
        let gutter_width = calculate_gutter_width(line_count);
        let content_width = inner.width.saturating_sub(gutter_width) as usize;

        if content_width == 0 {
            return;
        }

        // Ensure cursor is visible
        doc.ensure_cursor_visible(visible_lines, content_width);

        Some(DocRenderInfo {
            line_count,
            scroll_y: doc.scroll_y,
            scroll_x: doc.scroll_x,
            cursor_line: doc.cursor.line,
            cursor_col: doc.cursor.col,
            selection: doc.selection.clone(),
            gutter_width,
            content_width,
            lines: (0..visible_lines)
                .map(|row| {
                    let line_idx = doc.scroll_y + row;
                    if line_idx < line_count {
                        let content = doc
                            .buffer
                            .line(line_idx)
                            .map(|s| {
                                let s = s.to_string();
                                s.trim_end_matches('\n').trim_end_matches('\r').to_string()
                            })
                            .unwrap_or_default();
                        Some(content)
                    } else {
                        None
                    }
                })
                .collect(),
        })
    } else {
        None
    };

    let Some(info) = doc_info else {
        return;
    };

    let mut lines: Vec<Line> = Vec::new();

    // Render visible lines
    for (screen_row, line_content) in info.lines.iter().enumerate() {
        let line_idx = info.scroll_y + screen_row;

        if let Some(content) = line_content {
            let is_current_line = line_idx == info.cursor_line;

            // Line number
            let num_style = if is_current_line {
                Style::default()
                    .fg(theme.line_number_current)
                    .bg(theme.editor_bg)
            } else {
                Style::default().fg(theme.line_number).bg(theme.editor_bg)
            };

            let num_str = format!(
                "{:>width$} ",
                line_idx + 1,
                width = (info.gutter_width - 1) as usize
            );

            // Handle horizontal scrolling
            let display_start = info.scroll_x;
            let display_content: String = content
                .chars()
                .skip(display_start)
                .take(info.content_width)
                .collect();

            // Pad to fill width
            let padded_content = format!("{:<width$}", display_content, width = info.content_width);

            // Build spans with cursor and selection highlighting
            let mut spans = vec![Span::styled(num_str, num_style)];

            // Determine background for each character
            for (col_offset, ch) in padded_content.chars().enumerate() {
                let actual_col = info.scroll_x + col_offset;

                let is_cursor = focused && is_current_line && actual_col == info.cursor_col;
                let is_selected = info.selection.contains(line_idx, actual_col);

                let style = if is_cursor {
                    Style::default().fg(theme.editor_bg).bg(theme.cursor)
                } else if is_selected {
                    Style::default().fg(theme.fg).bg(theme.selection_bg)
                } else if is_current_line {
                    Style::default().fg(theme.fg).bg(theme.line_highlight)
                } else {
                    Style::default().fg(theme.fg).bg(theme.editor_bg)
                };

                spans.push(Span::styled(ch.to_string(), style));
            }

            lines.push(Line::from(spans));
        } else {
            // Empty line (past end of document)
            let num_style = Style::default().fg(theme.line_number).bg(theme.editor_bg);
            let code_style = Style::default().bg(theme.editor_bg);

            let num_str = format!("{:>width$} ", "~", width = (info.gutter_width - 1) as usize);

            lines.push(Line::from(vec![
                Span::styled(num_str, num_style),
                Span::styled(" ".repeat(info.content_width), code_style),
            ]));
        }
    }

    let content = Paragraph::new(lines);
    frame.render_widget(content, inner);
}

/// Information extracted from document for rendering
struct DocRenderInfo {
    line_count: usize,
    scroll_y: usize,
    scroll_x: usize,
    cursor_line: usize,
    cursor_col: usize,
    selection: crate::editor::Selection,
    gutter_width: u16,
    content_width: usize,
    lines: Vec<Option<String>>,
}

/// Calculate the width needed for line numbers
fn calculate_gutter_width(line_count: usize) -> u16 {
    let digits = if line_count == 0 {
        1
    } else {
        (line_count as f64).log10().floor() as u16 + 1
    };
    // At least 4 digits, plus 1 for space
    digits.max(4) + 1
}

/// Get the document position from screen coordinates
pub fn position_from_screen(app: &App, x: u16, y: u16) -> Option<(usize, usize)> {
    let area = app.editor_area?;

    if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
        return None;
    }

    let doc = app.active_document()?;
    let gutter_width = calculate_gutter_width(doc.line_count());

    // Check if click is in gutter
    if x < area.x + gutter_width {
        return None;
    }

    let screen_row = (y - area.y) as usize;
    let screen_col = (x - area.x - gutter_width) as usize;

    let line = doc.scroll_y + screen_row;
    let col = doc.scroll_x + screen_col;

    // Clamp to valid positions
    let line = line.min(doc.line_count().saturating_sub(1));
    let col = col.min(doc.line_len(line));

    Some((line, col))
}
