use crate::app::App;
use crate::file_tree::EntryKind;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Draw the file tree sidebar
pub fn draw(frame: &mut Frame, app: &mut App, area: Rect, focused: bool) {
    let border_color = if focused {
        app.theme.border_focused
    } else {
        app.theme.border
    };

    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(app.theme.sidebar_bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Store the content area for mouse hit detection
    if inner.height > 1 {
        app.file_tree_area = Some(Rect {
            x: inner.x,
            y: inner.y + 1, // Account for header
            width: inner.width,
            height: inner.height - 1,
        });
    }

    // Draw directory name as header
    let cwd_name = app.cwd.file_name().and_then(|n| n.to_str()).unwrap_or(".");

    let header = Paragraph::new(format!(" {} ", cwd_name)).style(
        Style::default()
            .fg(app.theme.tree_dir)
            .bg(app.theme.sidebar_bg)
            .add_modifier(Modifier::BOLD),
    );

    if inner.height > 0 {
        let header_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: 1,
        };
        frame.render_widget(header, header_area);
    }

    // Draw file tree entries
    if inner.height > 1 {
        let content_area = Rect {
            x: inner.x,
            y: inner.y + 1,
            width: inner.width,
            height: inner.height - 1,
        };

        let visible_height = content_area.height as usize;

        // Ensure selected item is visible
        app.file_tree.ensure_visible_with_height(visible_height);

        let mut lines: Vec<Line> = Vec::new();

        // Iterate over visible entries
        for i in 0..visible_height {
            let entry_idx = app.file_tree.scroll_offset + i;

            if let Some(entry) = app.file_tree.entries.get(entry_idx) {
                let is_selected = entry_idx == app.file_tree.selected;

                // Calculate indentation (2 spaces per depth level)
                let indent = "  ".repeat(entry.depth);

                // Icon based on type and expanded state
                let icon = match entry.kind {
                    EntryKind::Directory => {
                        if entry.expanded {
                            "▼ "
                        } else {
                            "▶ "
                        }
                    }
                    EntryKind::File => "  ",
                };

                // Color based on type
                let name_color = match entry.kind {
                    EntryKind::Directory => app.theme.tree_dir,
                    EntryKind::File => app.theme.tree_file,
                };

                // Background color for selection
                let bg_color = if is_selected {
                    app.theme.tree_selected_bg
                } else {
                    app.theme.sidebar_bg
                };

                // Build the display name
                let display_name = if entry.kind == EntryKind::Directory {
                    format!("{}/", entry.name)
                } else {
                    entry.name.clone()
                };

                // Truncate if needed
                let max_width = content_area.width.saturating_sub(1) as usize;
                let prefix = format!("{}{}", indent, icon);
                let available = max_width.saturating_sub(prefix.len());
                let truncated_name = if display_name.len() > available {
                    format!("{}…", &display_name[..available.saturating_sub(1)])
                } else {
                    display_name
                };

                // Pad to fill the width for proper background
                let full_text = format!("{}{}", prefix, truncated_name);
                let padded = format!("{:<width$}", full_text, width = max_width);

                let style = Style::default().fg(name_color).bg(bg_color);

                // Add modifier for selected item if focused
                let style = if is_selected && focused {
                    style.add_modifier(Modifier::BOLD)
                } else {
                    style
                };

                lines.push(Line::from(Span::styled(padded, style)));
            } else {
                // Empty line to fill the space
                let empty = " ".repeat(content_area.width.saturating_sub(1) as usize);
                lines.push(Line::from(Span::styled(
                    empty,
                    Style::default().bg(app.theme.sidebar_bg),
                )));
            }
        }

        let content = Paragraph::new(lines);
        frame.render_widget(content, content_area);
    }
}

/// Get the file tree entry index at a given screen position
pub fn entry_at_position(app: &App, x: u16, y: u16) -> Option<usize> {
    let area = app.file_tree_area?;

    // Check if within the file tree content area
    if x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height {
        let row = (y - area.y) as usize;
        app.file_tree.index_at_row(row)
    } else {
        None
    }
}
