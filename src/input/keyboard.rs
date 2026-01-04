use super::AppEvent;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Maps keyboard events to application events
pub fn map_key_event(key: KeyEvent) -> Option<AppEvent> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let alt = key.modifiers.contains(KeyModifiers::ALT);

    match key.code {
        // Quit: Ctrl+Q
        KeyCode::Char('q') if ctrl && !shift => Some(AppEvent::Quit),

        // File operations
        KeyCode::Char('n') if ctrl && !shift => Some(AppEvent::NewFile),
        KeyCode::Char('o') if ctrl && !shift => Some(AppEvent::OpenFile),
        KeyCode::Char('s') if ctrl && !shift => Some(AppEvent::Save),
        KeyCode::Char('s') | KeyCode::Char('S') if ctrl && shift => Some(AppEvent::SaveAs),
        KeyCode::Char('w') if ctrl && !shift => Some(AppEvent::CloseFile),
        KeyCode::Char('w') | KeyCode::Char('W') if ctrl && shift => Some(AppEvent::CloseAllFiles),

        // Edit operations
        KeyCode::Char('z') if ctrl && !shift => Some(AppEvent::Undo),
        KeyCode::Char('y') if ctrl => Some(AppEvent::Redo),
        KeyCode::Char('x') if ctrl => Some(AppEvent::Cut),
        KeyCode::Char('c') if ctrl => Some(AppEvent::Copy),
        KeyCode::Char('v') if ctrl => Some(AppEvent::Paste),
        KeyCode::Char('a') if ctrl => Some(AppEvent::SelectAll),
        KeyCode::Char('k') if ctrl => Some(AppEvent::DeleteLine),
        KeyCode::Char('d') if ctrl => Some(AppEvent::DuplicateLine),
        KeyCode::Up if alt => Some(AppEvent::MoveLineUp),
        KeyCode::Down if alt => Some(AppEvent::MoveLineDown),

        // Search
        KeyCode::Char('f') if ctrl => Some(AppEvent::Find),
        KeyCode::Char('g') if ctrl && !shift => Some(AppEvent::FindNext),
        KeyCode::Char('g') | KeyCode::Char('G') if ctrl && shift => Some(AppEvent::FindPrevious),
        KeyCode::F(3) if !shift => Some(AppEvent::FindNext),
        KeyCode::F(3) if shift => Some(AppEvent::FindPrevious),
        KeyCode::Char('h') if ctrl => Some(AppEvent::Replace),
        KeyCode::Char('l') if ctrl => Some(AppEvent::GoToLine),

        // Navigation
        KeyCode::Char('b') if ctrl => Some(AppEvent::GoToMatchingBrace),
        KeyCode::PageDown if ctrl => Some(AppEvent::NextTab),
        KeyCode::PageUp if ctrl => Some(AppEvent::PreviousTab),

        // Tab switching with Alt+1-9, Alt+0
        KeyCode::Char(c @ '1'..='9') if alt => {
            Some(AppEvent::GoToTab(c.to_digit(10).unwrap() as u8))
        }
        KeyCode::Char('0') if alt => Some(AppEvent::GoToTab(10)), // Last tab

        // Focus switching
        KeyCode::F(2) => Some(AppEvent::FocusEditor),
        KeyCode::F(4) => Some(AppEvent::FocusTerminal),

        // View toggles (Ctrl+Shift+B and Ctrl+Shift+T are handled specially)
        KeyCode::Char('B') if ctrl && shift => Some(AppEvent::ToggleSidebar),
        KeyCode::Char('T') if ctrl && shift => Some(AppEvent::ToggleTerminal),

        // Zoom
        KeyCode::Char('+') | KeyCode::Char('=') if ctrl => Some(AppEvent::ZoomIn),
        KeyCode::Char('-') if ctrl => Some(AppEvent::ZoomOut),
        KeyCode::Char('0') if ctrl => Some(AppEvent::ZoomReset),

        // Tab/BackTab for focus cycling
        KeyCode::Tab if !ctrl && !shift && !alt => Some(AppEvent::CycleFocusForward),
        KeyCode::BackTab => Some(AppEvent::CycleFocusBackward),

        _ => None,
    }
}
