use crate::app::App;
use crate::highlighting::{HighlightStyle, StyledSpan};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use syntect::easy::HighlightLines;

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

    // First pass: gather document info without highlighting
    let doc_info = {
        let doc = match app.active_document_mut() {
            Some(d) => d,
            None => return,
        };

        let line_count = doc.line_count();
        let gutter_width = calculate_gutter_width(line_count);
        let content_width = inner.width.saturating_sub(gutter_width) as usize;

        if content_width == 0 {
            return;
        }

        // Ensure cursor is visible
        doc.ensure_cursor_visible(visible_lines, content_width);

        // Collect basic info
        DocInfo {
            line_count,
            scroll_y: doc.scroll_y,
            scroll_x: doc.scroll_x,
            cursor_line: doc.cursor.line,
            cursor_col: doc.cursor.col,
            selection: doc.selection.clone(),
            gutter_width,
            content_width,
            filetype: doc.filetype.clone(),
        }
    };

    // Second pass: collect lines and do highlighting
    // We need to handle this carefully to work with borrow checker
    let mut line_data: Vec<LineRenderData> = Vec::new();

    for row in 0..visible_lines {
        let line_idx = doc_info.scroll_y + row;
        if line_idx >= doc_info.line_count {
            line_data.push(LineRenderData::Empty);
            continue;
        }

        // Get line content
        let content = app
            .active_document()
            .and_then(|doc| doc.buffer.line(line_idx))
            .map(|s| {
                let s = s.to_string();
                s.trim_end_matches('\n').trim_end_matches('\r').to_string()
            })
            .unwrap_or_default();

        // Try to get highlighted spans
        let highlighted_spans =
            highlight_line_content(&content, &doc_info.filetype, &app.highlighting);

        line_data.push(LineRenderData::Content {
            line_idx,
            content,
            highlighted_spans,
        });
    }

    // Now render everything
    let mut lines: Vec<Line> = Vec::new();

    for line_render in &line_data {
        match line_render {
            LineRenderData::Content {
                line_idx,
                content,
                highlighted_spans,
            } => {
                let is_current_line = *line_idx == doc_info.cursor_line;

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
                    width = (doc_info.gutter_width - 1) as usize
                );

                let mut spans = vec![Span::styled(num_str, num_style)];

                // Render with syntax highlighting if available
                if let Some(hl_spans) = highlighted_spans {
                    render_highlighted_line(
                        &mut spans, hl_spans, content, *line_idx, &doc_info, &theme, focused,
                    );
                } else {
                    // Fallback to plain rendering
                    render_plain_line(&mut spans, content, *line_idx, &doc_info, &theme, focused);
                }

                lines.push(Line::from(spans));
            }
            LineRenderData::Empty => {
                // Empty line (past end of document)
                let num_style = Style::default().fg(theme.line_number).bg(theme.editor_bg);
                let code_style = Style::default().bg(theme.editor_bg);

                let num_str = format!(
                    "{:>width$} ",
                    "~",
                    width = (doc_info.gutter_width - 1) as usize
                );

                lines.push(Line::from(vec![
                    Span::styled(num_str, num_style),
                    Span::styled(" ".repeat(doc_info.content_width), code_style),
                ]));
            }
        }
    }

    let content = Paragraph::new(lines);
    frame.render_widget(content, inner);
}

/// Highlight a single line of content
fn highlight_line_content(
    content: &str,
    filetype: &str,
    highlighting: &crate::highlighting::HighlightingManager,
) -> Option<Vec<StyledSpan>> {
    // Get syntax and theme
    let syntax = highlighting.syntax_for_filetype(filetype)?;
    let theme = highlighting.current_theme()?;

    // Use HighlightLines for simple line-by-line highlighting
    let mut highlighter = HighlightLines::new(syntax, theme);
    let ranges = highlighter
        .highlight_line(content, &highlighting.syntax_set)
        .ok()?;

    let spans: Vec<StyledSpan> = ranges
        .into_iter()
        .map(|(style, text)| StyledSpan {
            text: text.to_string(),
            style: HighlightStyle {
                fg: Some(Color::Rgb(
                    style.foreground.r,
                    style.foreground.g,
                    style.foreground.b,
                )),
                bg: if style.background.a > 0 {
                    Some(Color::Rgb(
                        style.background.r,
                        style.background.g,
                        style.background.b,
                    ))
                } else {
                    None
                },
                bold: style
                    .font_style
                    .contains(syntect::highlighting::FontStyle::BOLD),
                italic: style
                    .font_style
                    .contains(syntect::highlighting::FontStyle::ITALIC),
                underline: style
                    .font_style
                    .contains(syntect::highlighting::FontStyle::UNDERLINE),
            },
        })
        .collect();

    Some(spans)
}

/// Render a line with syntax highlighting
fn render_highlighted_line(
    spans: &mut Vec<Span<'static>>,
    hl_spans: &[StyledSpan],
    content: &str,
    line_idx: usize,
    info: &DocInfo,
    theme: &crate::theme::Theme,
    focused: bool,
) {
    let is_current_line = line_idx == info.cursor_line;

    // Build a character-level style map from the highlighted spans
    let chars: Vec<char> = content.chars().collect();
    let mut char_styles: Vec<HighlightStyle> = vec![HighlightStyle::default(); chars.len()];

    let mut char_idx = 0;
    for hl_span in hl_spans {
        for _ in hl_span.text.chars() {
            if char_idx < char_styles.len() {
                char_styles[char_idx] = hl_span.style;
            }
            char_idx += 1;
        }
    }

    // Render visible portion with cursor/selection overlay
    let display_start = info.scroll_x;

    for col_offset in 0..info.content_width {
        let actual_col = display_start + col_offset;
        let ch = if actual_col < chars.len() {
            chars[actual_col]
        } else {
            ' '
        };

        let is_cursor = focused && is_current_line && actual_col == info.cursor_col;
        let is_selected = info.selection.contains(line_idx, actual_col);

        // Get base style from highlighting
        let hl_style = if actual_col < char_styles.len() {
            char_styles[actual_col]
        } else {
            HighlightStyle::default()
        };

        // Build final style with cursor/selection/current line overlay
        let style = if is_cursor {
            Style::default().fg(theme.editor_bg).bg(theme.cursor)
        } else if is_selected {
            // Keep syntax color for foreground, use selection background
            let fg = hl_style.fg.unwrap_or(theme.fg);
            Style::default().fg(fg).bg(theme.selection_bg)
        } else if is_current_line {
            // Keep syntax color for foreground, use line highlight background
            let fg = hl_style.fg.unwrap_or(theme.fg);
            let mut s = Style::default().fg(fg).bg(theme.line_highlight);
            if hl_style.bold {
                s = s.add_modifier(Modifier::BOLD);
            }
            if hl_style.italic {
                s = s.add_modifier(Modifier::ITALIC);
            }
            s
        } else {
            // Full syntax highlighting
            let fg = hl_style.fg.unwrap_or(theme.fg);
            let bg = hl_style.bg.unwrap_or(theme.editor_bg);
            let mut s = Style::default().fg(fg).bg(bg);
            if hl_style.bold {
                s = s.add_modifier(Modifier::BOLD);
            }
            if hl_style.italic {
                s = s.add_modifier(Modifier::ITALIC);
            }
            s
        };

        spans.push(Span::styled(ch.to_string(), style));
    }
}

/// Render a line without syntax highlighting (fallback)
fn render_plain_line(
    spans: &mut Vec<Span<'static>>,
    content: &str,
    line_idx: usize,
    info: &DocInfo,
    theme: &crate::theme::Theme,
    focused: bool,
) {
    let is_current_line = line_idx == info.cursor_line;

    // Handle horizontal scrolling
    let display_start = info.scroll_x;
    let display_content: String = content
        .chars()
        .skip(display_start)
        .take(info.content_width)
        .collect();

    // Pad to fill width
    let padded_content = format!("{:<width$}", display_content, width = info.content_width);

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
}

/// Line render data
enum LineRenderData {
    Content {
        line_idx: usize,
        content: String,
        highlighted_spans: Option<Vec<StyledSpan>>,
    },
    Empty,
}

/// Information extracted from document for rendering
struct DocInfo {
    line_count: usize,
    scroll_y: usize,
    scroll_x: usize,
    cursor_line: usize,
    cursor_col: usize,
    selection: crate::editor::Selection,
    gutter_width: u16,
    content_width: usize,
    filetype: String,
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
