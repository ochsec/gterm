use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

/// Represents a processed mouse action
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseAction {
    /// Single click at position
    Click { x: u16, y: u16 },
    /// Double click at position (select word)
    DoubleClick { x: u16, y: u16 },
    /// Triple click at position (select line)
    TripleClick { x: u16, y: u16 },
    /// Start dragging from position
    DragStart { x: u16, y: u16 },
    /// Continue dragging to position
    Drag { x: u16, y: u16 },
    /// Stop dragging at position
    DragEnd { x: u16, y: u16 },
    /// Right click (context menu)
    RightClick { x: u16, y: u16 },
    /// Middle click (often paste or close tab)
    MiddleClick { x: u16, y: u16 },
    /// Scroll up
    ScrollUp { x: u16, y: u16, amount: u16 },
    /// Scroll down
    ScrollDown { x: u16, y: u16, amount: u16 },
}

/// Maps a raw mouse event to a mouse action
pub fn map_mouse_event(event: MouseEvent) -> Option<MouseAction> {
    let x = event.column;
    let y = event.row;

    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => Some(MouseAction::Click { x, y }),
        MouseEventKind::Down(MouseButton::Right) => Some(MouseAction::RightClick { x, y }),
        MouseEventKind::Down(MouseButton::Middle) => Some(MouseAction::MiddleClick { x, y }),
        MouseEventKind::Up(MouseButton::Left) => Some(MouseAction::DragEnd { x, y }),
        MouseEventKind::Drag(MouseButton::Left) => Some(MouseAction::Drag { x, y }),
        MouseEventKind::ScrollUp => Some(MouseAction::ScrollUp { x, y, amount: 3 }),
        MouseEventKind::ScrollDown => Some(MouseAction::ScrollDown { x, y, amount: 3 }),
        _ => None,
    }
}
