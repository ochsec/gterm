use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

/// Represents an entry in the file tree (file or directory)
#[derive(Debug, Clone)]
pub struct FileTreeEntry {
    /// The file/directory name
    pub name: String,
    /// Full path to the entry
    pub path: PathBuf,
    /// Whether this is a file or directory
    pub kind: EntryKind,
    /// Depth level in the tree (0 = root)
    pub depth: usize,
    /// Whether this directory is expanded (only relevant for directories)
    pub expanded: bool,
    /// Children entries (only populated when expanded)
    pub children: Vec<FileTreeEntry>,
}

/// The type of file tree entry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
}

/// The file tree state
#[derive(Debug)]
pub struct FileTree {
    /// Root directory path
    pub root: PathBuf,
    /// All visible entries (flattened for display)
    pub entries: Vec<FileTreeEntry>,
    /// Currently selected index
    pub selected: usize,
    /// Scroll offset for display
    pub scroll_offset: usize,
    /// Whether to show hidden files
    pub show_hidden: bool,
}

impl FileTreeEntry {
    /// Create a new entry from a path
    pub fn from_path(path: &Path, depth: usize) -> Option<Self> {
        let name = path.file_name()?.to_str()?.to_string();
        let metadata = fs::metadata(path).ok()?;

        let kind = if metadata.is_dir() {
            EntryKind::Directory
        } else {
            EntryKind::File
        };

        Some(Self {
            name,
            path: path.to_path_buf(),
            kind,
            depth,
            expanded: false,
            children: Vec::new(),
        })
    }

    /// Check if this is a directory
    pub fn is_dir(&self) -> bool {
        self.kind == EntryKind::Directory
    }

    /// Load children for a directory entry
    pub fn load_children(&mut self, show_hidden: bool) {
        if self.kind != EntryKind::Directory {
            return;
        }

        self.children.clear();

        if let Ok(entries) = fs::read_dir(&self.path) {
            let mut children: Vec<FileTreeEntry> = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let path = e.path();
                    let name = path.file_name()?.to_str()?;

                    // Skip hidden files if not showing them
                    if !show_hidden && name.starts_with('.') {
                        return None;
                    }

                    FileTreeEntry::from_path(&path, self.depth + 1)
                })
                .collect();

            // Sort: directories first, then alphabetically (case-insensitive)
            children.sort_by(|a, b| match (a.kind, b.kind) {
                (EntryKind::Directory, EntryKind::File) => Ordering::Less,
                (EntryKind::File, EntryKind::Directory) => Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            });

            self.children = children;
        }
    }
}

impl FileTree {
    /// Create a new file tree rooted at the given path
    pub fn new(root: PathBuf, show_hidden: bool) -> Self {
        let mut tree = Self {
            root: root.clone(),
            entries: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            show_hidden,
        };

        tree.refresh();
        tree
    }

    /// Refresh the file tree from disk
    pub fn refresh(&mut self) {
        self.entries.clear();

        // Create root entry
        if let Some(mut root_entry) = FileTreeEntry::from_path(&self.root, 0) {
            root_entry.expanded = true; // Root is always expanded
            root_entry.load_children(self.show_hidden);

            // Flatten the tree for display
            self.flatten_entry(&root_entry);
        }

        // Ensure selected index is valid
        if self.selected >= self.entries.len() && !self.entries.is_empty() {
            self.selected = self.entries.len() - 1;
        }
    }

    /// Flatten an entry and its expanded children into the entries list
    fn flatten_entry(&mut self, entry: &FileTreeEntry) {
        self.entries.push(entry.clone());

        if entry.expanded {
            for child in &entry.children {
                self.flatten_entry(child);
            }
        }
    }

    /// Get the currently selected entry
    pub fn selected_entry(&self) -> Option<&FileTreeEntry> {
        self.entries.get(self.selected)
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.ensure_visible();
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
            self.ensure_visible();
        }
    }

    /// Toggle expand/collapse of the selected directory
    pub fn toggle_expand(&mut self) {
        if let Some(entry) = self.entries.get(self.selected) {
            if entry.kind == EntryKind::Directory {
                let path = entry.path.clone();
                let was_expanded = entry.expanded;

                // Find and update the entry in the tree
                self.toggle_entry_expanded(&path, !was_expanded);

                // Rebuild the flattened list
                self.rebuild_entries();
            }
        }
    }

    /// Toggle the expanded state of an entry by path
    fn toggle_entry_expanded(&mut self, target_path: &Path, expanded: bool) {
        // We need to find the entry and modify it
        // Since entries is a flattened list, we need to rebuild
        for entry in &mut self.entries {
            if entry.path == target_path {
                entry.expanded = expanded;
                if expanded && entry.children.is_empty() {
                    entry.load_children(self.show_hidden);
                }
                break;
            }
        }
    }

    /// Rebuild the entries list from the current tree state
    fn rebuild_entries(&mut self) {
        // Store expanded states
        let expanded_paths: std::collections::HashSet<PathBuf> = self
            .entries
            .iter()
            .filter(|e| e.expanded)
            .map(|e| e.path.clone())
            .collect();

        // Refresh from disk
        self.entries.clear();

        if let Some(mut root_entry) = FileTreeEntry::from_path(&self.root, 0) {
            root_entry.expanded = true;
            self.rebuild_entry(&mut root_entry, &expanded_paths);
            self.flatten_entry(&root_entry);
        }

        // Ensure selected index is valid
        if self.selected >= self.entries.len() && !self.entries.is_empty() {
            self.selected = self.entries.len() - 1;
        }
    }

    /// Recursively rebuild an entry, restoring expanded states
    fn rebuild_entry(
        &self,
        entry: &mut FileTreeEntry,
        expanded_paths: &std::collections::HashSet<PathBuf>,
    ) {
        if entry.kind == EntryKind::Directory {
            entry.expanded = expanded_paths.contains(&entry.path);

            if entry.expanded {
                entry.load_children(self.show_hidden);

                for child in &mut entry.children {
                    self.rebuild_entry(child, expanded_paths);
                }
            }
        }
    }

    /// Ensure the selected item is visible in the viewport
    fn ensure_visible(&mut self) {
        // This will be used with the visible_height parameter from rendering
        // For now, just ensure scroll_offset doesn't exceed selected
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
    }

    /// Adjust scroll to ensure selected item is visible given viewport height
    pub fn ensure_visible_with_height(&mut self, visible_height: usize) {
        if visible_height == 0 {
            return;
        }

        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + visible_height {
            self.scroll_offset = self.selected - visible_height + 1;
        }
    }

    /// Handle page up
    pub fn page_up(&mut self, page_size: usize) {
        if self.selected > page_size {
            self.selected -= page_size;
        } else {
            self.selected = 0;
        }
        self.ensure_visible();
    }

    /// Handle page down
    pub fn page_down(&mut self, page_size: usize) {
        self.selected = (self.selected + page_size).min(self.entries.len().saturating_sub(1));
        self.ensure_visible();
    }

    /// Go to the first entry
    pub fn go_to_top(&mut self) {
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Go to the last entry
    pub fn go_to_bottom(&mut self) {
        if !self.entries.is_empty() {
            self.selected = self.entries.len() - 1;
        }
        self.ensure_visible();
    }

    /// Select an entry by index (from mouse click)
    pub fn select_index(&mut self, index: usize) {
        if index < self.entries.len() {
            self.selected = index;
        }
    }

    /// Get the entry at a given visual row (accounting for scroll)
    pub fn entry_at_row(&self, row: usize) -> Option<&FileTreeEntry> {
        let index = self.scroll_offset + row;
        self.entries.get(index)
    }

    /// Get the index at a given visual row
    pub fn index_at_row(&self, row: usize) -> Option<usize> {
        let index = self.scroll_offset + row;
        if index < self.entries.len() {
            Some(index)
        } else {
            None
        }
    }
}
