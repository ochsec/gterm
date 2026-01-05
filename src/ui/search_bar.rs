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

    let bg_style = Style::default()
        .bg(app.theme.statusbar_bg)
        .fg(app.theme.statusbar_fg);

    let dim_style = Style::default()
        .fg(app.theme.line_number)
        .bg(app.theme.statusbar_bg);

    if app.search.replace_mode {
        // Replace mode: 2 lines
        draw_replace_bar(frame, app, area, bg_style, dim_style);
    } else {
        // Find mode: 1 line
        draw_find_bar(frame, app, area, bg_style, dim_style);
    }
}

/// Draw the simple find bar (1 line)
fn draw_find_bar(frame: &mut Frame, app: &App, area: Rect, bg_style: Style, dim_style: Style) {
    let search_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    };

    let query = &app.search.query;
    let match_info = app.search.match_info();

    // Create spans for the search bar
    let spans = vec![
        Span::styled("Find: ", bg_style.add_modifier(Modifier::BOLD)),
        Span::styled(query.as_str(), bg_style),
        Span::styled(
            "|",
            Style::default().fg(app.theme.fg).bg(app.theme.statusbar_bg),
        ), // Cursor
        Span::styled(format!(" {}", match_info), dim_style),
    ];

    // Calculate padding to fill the width
    let content_width = 6 + query.len() + 1 + match_info.len() + 1;
    let hint = " [Enter:next Shift+F3:prev Esc:close]";

    let mut all_spans = spans;

    // Add padding and hint if room
    if area.width as usize > content_width + hint.len() {
        let padding = area
            .width
            .saturating_sub((content_width + hint.len()) as u16) as usize;
        all_spans.push(Span::styled(" ".repeat(padding), bg_style));
        all_spans.push(Span::styled(hint, dim_style));
    }

    let line = Line::from(all_spans);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, search_area);
}

/// Draw the replace bar (2 lines)
fn draw_replace_bar(frame: &mut Frame, app: &App, area: Rect, bg_style: Style, dim_style: Style) {
    // Line 1: Find field
    let find_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    };

    // Line 2: Replace field
    let replace_area = Rect {
        x: area.x,
        y: area.y + 1,
        width: area.width,
        height: 1,
    };

    let query = &app.search.query;
    let replace = &app.search.replace_text;
    let match_info = app.search.match_info();
    let focus = app.search.replace_focus;

    // Style for focused vs unfocused fields
    let find_style = if focus == 0 { bg_style } else { dim_style };
    let replace_style = if focus == 1 { bg_style } else { dim_style };

    // Find line
    let find_cursor = if focus == 0 { "|" } else { "" };
    let find_spans = vec![
        Span::styled("Find:    ", find_style.add_modifier(Modifier::BOLD)),
        Span::styled(query.as_str(), find_style),
        Span::styled(
            find_cursor,
            Style::default().fg(app.theme.fg).bg(app.theme.statusbar_bg),
        ),
        Span::styled(format!(" {}", match_info), dim_style),
    ];

    // Calculate padding for find line
    let find_content_width = 9 + query.len() + find_cursor.len() + match_info.len() + 1;
    let hint = " [Tab:switch Enter:replace Ctrl+A:all Esc:close]";

    let mut find_all_spans = find_spans;
    if area.width as usize > find_content_width + hint.len() {
        let padding =
            area.width
                .saturating_sub((find_content_width + hint.len()) as u16) as usize;
        find_all_spans.push(Span::styled(" ".repeat(padding), bg_style));
        find_all_spans.push(Span::styled(hint, dim_style));
    }

    let find_line = Line::from(find_all_spans);
    frame.render_widget(Paragraph::new(find_line), find_area);

    // Replace line
    let replace_cursor = if focus == 1 { "|" } else { "" };
    let replace_spans = vec![
        Span::styled("Replace: ", replace_style.add_modifier(Modifier::BOLD)),
        Span::styled(replace.as_str(), replace_style),
        Span::styled(
            replace_cursor,
            Style::default().fg(app.theme.fg).bg(app.theme.statusbar_bg),
        ),
    ];

    // Fill the rest with background
    let replace_content_width = 9 + replace.len() + replace_cursor.len();
    let padding = area.width.saturating_sub(replace_content_width as u16) as usize;

    let mut replace_all_spans = replace_spans;
    replace_all_spans.push(Span::styled(" ".repeat(padding), bg_style));

    let replace_line = Line::from(replace_all_spans);
    frame.render_widget(Paragraph::new(replace_line), replace_area);
}

/// Calculate the height of the search bar (0 if not active, 1 for find, 2 for replace)
pub fn height(app: &App) -> u16 {
    if !app.search.active {
        0
    } else if app.search.replace_mode {
        2
    } else {
        1
    }
}
