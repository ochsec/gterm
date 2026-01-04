# gterm - Complete Project Specification

A TUI (Terminal User Interface) code editor inspired by Geany, written in Rust.

## Project Overview

**Name:** gterm  
**Description:** A TUI code editor inspired by Geany, built with Rust and ratatui  
**License:** GPL-2.0 (matching Geany)

---

## Technical Decisions

| Decision | Choice |
|----------|--------|
| Project Name | `gterm` |
| Text Buffer | Rope (`ropey` crate) |
| Config Format | TOML |
| Default Shell | `$SHELL` env var |
| Build Menu | Deferred |
| Plugin System | Not included |
| Minimum Size | 80×24 characters |
| Mouse Support | Full (drag select, resize panes, context menus) |
| Theme | Dark (Geany-inspired) |
| File Tree Position | Left side |
| Status Bar | Full Geany-style |

---

## Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│ File  Edit  Search  View  Document  Help                    [gterm] │
├──────────────┬──────────────────────────────────────────────────────┤
│              │  tab1.rs │ tab2.rs │ tab3.rs* │                      │
│  File Tree   ├──────────────────────────────────────────────────────┤
│  (left)      │                                                      │
│              │                  Code Editor                         │
│  ▼ src/      │                  (top-middle)                        │
│    main.rs   │                                                      │
│    app.rs    │   1 │ fn main() {                                    │
│  ▼ ui/       │   2 │     println!("Hello");                         │
│    mod.rs    │   3 │ }                                              │
│              │                                                      │
│              ├──────────────────────────────────────────────────────┤
│              │ $ cargo run                                          │
│              │ Compiling gterm v0.1.0                               │
│              │     Finished dev [unoptimized] target(s)             │
│              │ $ █                                                  │
│              │                  Terminal (bottom-middle)            │
├──────────────┴──────────────────────────────────────────────────────┤
│ line: 2/3  col: 5  sel: 0  INS  SP  EOL: LF  UTF-8  Rust  main()   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Crate Dependencies

```toml
[dependencies]
# TUI
ratatui = "0.28"
crossterm = "0.28"

# Async runtime
tokio = { version = "1", features = ["full"] }

# Text editing
ropey = "1.6"           # Rope data structure
syntect = "5"           # Syntax highlighting
unicode-width = "0.1"   # Unicode character widths
unicode-segmentation = "1.10"

# Terminal emulation
portable-pty = "0.8"    # PTY spawning
vt100 = "0.15"          # VT100 parser

# File operations
ignore = "0.4"          # Directory traversal (respects .gitignore)
notify = "6"            # File watching

# Clipboard
arboard = "3"           # Cross-platform clipboard

# Config
serde = { version = "1", features = ["derive"] }
toml = "0.8"
dirs = "5"              # XDG directories

# Error handling
anyhow = "1"
thiserror = "1"

# Logging (for debugging)
log = "0.4"
env_logger = "0.11"
```

---

## File Structure

```
gterm/
├── Cargo.toml
├── README.md
├── LICENSE                    # GPL-2.0 (matching Geany)
├── config/
│   └── default.toml           # Default configuration
├── themes/
│   └── dark.toml              # Dark theme colors
└── src/
    ├── main.rs                # Entry point
    ├── app.rs                 # Application state & event loop
    ├── config.rs              # Configuration loading
    ├── theme.rs               # Theme definitions
    │
    ├── ui/
    │   ├── mod.rs
    │   ├── layout.rs          # Three-pane layout management
    │   ├── menu_bar.rs        # Top menu bar
    │   ├── tab_bar.rs         # Document tabs
    │   ├── editor.rs          # Editor widget rendering
    │   ├── file_tree.rs       # File tree widget
    │   ├── terminal.rs        # Terminal widget rendering
    │   └── status_bar.rs      # Bottom status bar
    │
    ├── editor/
    │   ├── mod.rs
    │   ├── buffer.rs          # Rope-based text buffer
    │   ├── cursor.rs          # Cursor position & selection
    │   ├── document.rs        # Document (buffer + metadata)
    │   ├── syntax.rs          # Syntect integration
    │   ├── history.rs         # Undo/redo stack
    │   └── commands.rs        # Editor commands/actions
    │
    ├── terminal/
    │   ├── mod.rs
    │   ├── pty.rs             # PTY process management
    │   └── screen.rs          # VT100 screen buffer
    │
    ├── file_tree/
    │   ├── mod.rs
    │   └── tree.rs            # Directory tree data structure
    │
    ├── input/
    │   ├── mod.rs
    │   ├── event.rs           # Event types
    │   ├── keyboard.rs        # Keyboard handling
    │   └── mouse.rs           # Mouse handling
    │
    └── utils/
        ├── mod.rs
        └── clipboard.rs       # Clipboard operations
```

---

## Keyboard Shortcuts

### File
| Action | Shortcut |
|--------|----------|
| New | `Ctrl+N` |
| Open | `Ctrl+O` |
| Save | `Ctrl+S` |
| Save As | `Ctrl+Shift+S` |
| Save All | `Ctrl+Shift+A` |
| Close | `Ctrl+W` |
| Close All | `Ctrl+Shift+W` |
| Quit | `Ctrl+Q` |

### Edit
| Action | Shortcut |
|--------|----------|
| Undo | `Ctrl+Z` |
| Redo | `Ctrl+Y` |
| Cut | `Ctrl+X` |
| Copy | `Ctrl+C` |
| Paste | `Ctrl+V` |
| Select All | `Ctrl+A` |
| Delete Line | `Ctrl+K` |
| Duplicate Line | `Ctrl+D` |
| Move Line Up | `Alt+Up` |
| Move Line Down | `Alt+Down` |

### Search
| Action | Shortcut |
|--------|----------|
| Find | `Ctrl+F` |
| Find Next | `Ctrl+G` / `F3` |
| Find Previous | `Ctrl+Shift+G` / `Shift+F3` |
| Replace | `Ctrl+H` |
| Go to Line | `Ctrl+L` |

### Navigation
| Action | Shortcut |
|--------|----------|
| Go to Matching Brace | `Ctrl+B` |
| Next Tab | `Ctrl+PageDown` |
| Previous Tab | `Ctrl+PageUp` |
| Switch to Tab N | `Alt+1` through `Alt+9` |
| Last Tab | `Alt+0` |

### View/Focus
| Action | Shortcut |
|--------|----------|
| Focus Editor | `F2` |
| Focus Terminal | `F4` |
| Focus File Tree | `F3` |
| Toggle Sidebar | `Ctrl+Shift+B` |
| Toggle Terminal | `Ctrl+Shift+T` |
| Zoom In | `Ctrl++` |
| Zoom Out | `Ctrl+-` |
| Reset Zoom | `Ctrl+0` |

---

## Mouse Support

| Action | Mouse Event |
|--------|-------------|
| Position cursor | Left click in editor |
| Select text | Left click + drag in editor |
| Extend selection | Shift + left click |
| Select word | Double click |
| Select line | Triple click |
| Open file | Left click on file tree item |
| Expand/collapse dir | Left click on directory |
| Switch tab | Left click on tab |
| Close tab | Middle click on tab / click × |
| Resize panes | Drag pane dividers |
| Scroll | Scroll wheel |
| Context menu | Right click |
| Terminal input | Click in terminal to focus |

---

## Implementation Phases

### Phase 1: Foundation (Core App Structure)
1. Project setup with Cargo
2. Basic ratatui app with event loop
3. Three-pane layout with static content
4. Focus management (which pane is active)
5. Pane resizing with mouse drag

### Phase 2: File Tree
6. Directory tree data structure
7. File tree rendering
8. Keyboard navigation (arrows, Enter)
9. Mouse click to select
10. Expand/collapse directories

### Phase 3: Basic Editor
11. Rope-based text buffer
12. Document struct (buffer + cursor + metadata)
13. Editor rendering with line numbers
14. Cursor movement (arrows, Home, End, PageUp/Down)
15. Basic text input (insert characters)
16. Backspace/Delete
17. Mouse click to position cursor

### Phase 4: Editor Features
18. Text selection (keyboard: Shift+arrows)
19. Mouse text selection (click + drag)
20. Cut/Copy/Paste with clipboard
21. Undo/Redo
22. Multiple documents with tabs
23. Tab switching

### Phase 5: Syntax Highlighting
24. Syntect integration
25. Filetype detection
26. Dark theme colors
27. Line-by-line highlighting

### Phase 6: Terminal
28. PTY spawning with $SHELL
29. VT100 screen buffer
30. Terminal rendering
31. Keyboard input to PTY
32. Mouse selection in terminal
33. Scrollback buffer

### Phase 7: File Operations
34. Open file (Ctrl+O dialog)
35. Save file (Ctrl+S)
36. Save As (Ctrl+Shift+S)
37. New file (Ctrl+N)
38. Close file (Ctrl+W)
39. File change detection

### Phase 8: Search
40. Find dialog (Ctrl+F)
41. Find Next/Previous
42. Replace dialog (Ctrl+H)
43. Go to Line (Ctrl+L)

### Phase 9: Polish
44. Menu bar (clickable)
45. Status bar (full info)
46. Configuration file support
47. Bracket matching
48. Auto-indentation
49. Recent files

---

## Status Bar Format

```
line: {line}/{total_lines}  col: {col}  sel: {selection_chars}  {INS|OVR}  {TAB|SP}  {MOD}  EOL: {LF|CRLF|CR}  {encoding}  {filetype}  {scope}
```

Example:
```
line: 42/156  col: 15  sel: 0  INS  SP  EOL: LF  UTF-8  Rust  main()
```

---

## Configuration File (`~/.config/gterm/config.toml`)

```toml
[editor]
font_size = 1                    # Zoom level (0 = default)
tab_width = 4
insert_spaces = true             # Use spaces instead of tabs
auto_indent = true
show_line_numbers = true
highlight_current_line = true
word_wrap = false

[terminal]
shell = ""                       # Empty = use $SHELL

[ui]
show_sidebar = true
show_terminal = true
sidebar_width = 25               # Percentage
terminal_height = 30             # Percentage
theme = "dark"

[file_tree]
show_hidden = false
follow_current_file = true

[keybindings]
# Override default keybindings here
# find = "Ctrl+F"
```

---

## Milestone Summary

- **After Phase 3**: Basic usable text editor
- **After Phase 5**: Syntax-highlighted editor
- **After Phase 6**: Full IDE-like experience with terminal
- **After Phase 9**: Feature-complete Geany TUI clone

Estimated scope: ~4,000-6,000 lines of Rust code.
