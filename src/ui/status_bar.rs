use crate::app::App;
use ratatui::{prelude::*, widgets::Paragraph};

/// Draw the status bar at the bottom of the screen
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let style = Style::default()
        .fg(app.theme.statusbar_fg)
        .bg(app.theme.statusbar_bg);

    // Get info from active document
    let (line, total_lines, col, selection_len, insert_mode, eol, encoding, filetype) =
        if let Some(doc) = app.active_document() {
            let sel_len = if doc.selection.has_selection() {
                doc.selected_text().len()
            } else {
                0
            };

            (
                doc.cursor.line + 1, // 1-based for display
                doc.line_count(),
                doc.cursor.col + 1, // 1-based for display
                sel_len,
                if doc.insert_mode { "INS" } else { "OVR" },
                doc.line_ending.display_name(),
                doc.encoding.as_str(),
                doc.filetype.as_str(),
            )
        } else {
            (1, 1, 1, 0, "INS", "LF", "UTF-8", "Plain Text")
        };

    let indent_mode = "SP"; // Spaces (we hardcoded 4 spaces for tabs)
    let modified = app.active_document().map(|d| d.modified).unwrap_or(false);
    let mod_indicator = if modified { " [+]" } else { "" };

    let left_status = format!(
        " line: {}/{} | col: {} | sel: {} | {} | {} | EOL: {} | {} | {}{}",
        line,
        total_lines,
        col,
        selection_len,
        insert_mode,
        indent_mode,
        eol,
        encoding,
        filetype,
        mod_indicator
    );

    // For scope, we could show function name, but that requires parsing
    // For now, just show the focused pane
    let pane_name = match app.focused_pane {
        crate::ui::Pane::FileTree => "Files",
        crate::ui::Pane::Editor => "Editor",
        crate::ui::Pane::Terminal => "Terminal",
    };
    let right_status = format!(" {} ", pane_name);

    // Calculate padding
    let total_len = left_status.len() + right_status.len();
    let padding = area.width.saturating_sub(total_len as u16);

    let full_text = format!(
        "{}{}{}",
        left_status,
        " ".repeat(padding as usize),
        right_status
    );

    let status = Paragraph::new(full_text).style(style);
    frame.render_widget(status, area);
}
