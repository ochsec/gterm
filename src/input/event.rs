/// Application-level events (commands/actions)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    // App control
    Quit,

    // Focus control
    FocusFileTree,
    FocusEditor,
    FocusTerminal,
    CycleFocusForward,
    CycleFocusBackward,

    // View toggles
    ToggleSidebar,
    ToggleTerminal,

    // File operations
    NewFile,
    OpenFile,
    Save,
    SaveAs,
    SaveAll,
    CloseFile,
    CloseAllFiles,

    // Edit operations
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    DeleteLine,
    DuplicateLine,
    MoveLineUp,
    MoveLineDown,

    // Search
    Find,
    FindNext,
    FindPrevious,
    Replace,
    GoToLine,

    // Navigation
    GoToMatchingBrace,
    NextTab,
    PreviousTab,
    GoToTab(u8),

    // Editor zoom
    ZoomIn,
    ZoomOut,
    ZoomReset,
}
