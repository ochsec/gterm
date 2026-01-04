use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph},
};

/// Draw the menu bar at the top of the screen
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let menu_items = vec![
        " File ",
        " Edit ",
        " Search ",
        " View ",
        " Document ",
        " Help ",
    ];

    let menu_text: String = menu_items.join(" ");

    // Calculate padding to right-align the app name
    let title = "gterm";
    let padding = area
        .width
        .saturating_sub(menu_text.len() as u16 + title.len() as u16 + 2);
    let padded_title = format!(
        "{:>width$}",
        title,
        width = (padding + title.len() as u16) as usize
    );

    let full_text = format!("{}{}", menu_text, padded_title);

    let style = Style::default()
        .fg(app.theme.menubar_fg)
        .bg(app.theme.menubar_bg);

    let menu = Paragraph::new(full_text).style(style);

    frame.render_widget(menu, area);
}
