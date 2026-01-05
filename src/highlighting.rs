//! Syntax highlighting module using syntect
//!
//! This module provides syntax highlighting capabilities for the editor.
//! It supports loading syntax definitions and themes from:
//! - Built-in defaults (syntect's default newlines package)
//! - User directory: ~/.config/gterm/syntaxes/ for .sublime-syntax files
//! - User directory: ~/.config/gterm/themes/ for .tmTheme files

use ratatui::style::Color;
use std::collections::HashMap;
use std::path::PathBuf;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

/// Manages syntax highlighting resources
pub struct HighlightingManager {
    /// Syntax definitions
    pub syntax_set: SyntaxSet,
    /// Color themes
    pub theme_set: ThemeSet,
    /// Currently active theme name
    pub current_theme: String,
    /// Cache mapping filetype names to syntax references
    filetype_cache: HashMap<String, String>, // filetype -> syntax name
}

impl HighlightingManager {
    /// Create a new highlighting manager with default and user syntaxes/themes
    pub fn new() -> Self {
        let mut syntax_set = SyntaxSet::load_defaults_newlines();
        let mut theme_set = ThemeSet::load_defaults();

        // Try to load user syntaxes and themes
        if let Some(config_dir) = Self::config_dir() {
            // Load user syntaxes
            let syntaxes_dir = config_dir.join("syntaxes");
            if syntaxes_dir.exists() {
                let mut builder = syntax_set.into_builder();
                if let Ok(entries) = std::fs::read_dir(&syntaxes_dir) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.extension().map_or(false, |e| e == "sublime-syntax") {
                            if let Ok(syntax) = syntect::parsing::SyntaxDefinition::load_from_str(
                                &std::fs::read_to_string(&path).unwrap_or_default(),
                                true,
                                path.file_stem().and_then(|s| s.to_str()),
                            ) {
                                builder.add(syntax);
                            }
                        }
                    }
                }
                syntax_set = builder.build();
            }

            // Load user themes
            let themes_dir = config_dir.join("themes");
            if themes_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&themes_dir) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.extension().map_or(false, |e| e == "tmTheme") {
                            if let Ok(theme) = ThemeSet::get_theme(&path) {
                                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                                    theme_set.themes.insert(name.to_string(), theme);
                                }
                            }
                        }
                    }
                }
            }
        }

        Self {
            syntax_set,
            theme_set,
            current_theme: "base16-ocean.dark".to_string(),
            filetype_cache: Self::build_filetype_cache(),
        }
    }

    /// Get the config directory for gterm
    fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("gterm"))
    }

    /// Build a cache mapping our filetype names to syntect syntax names
    fn build_filetype_cache() -> HashMap<String, String> {
        let mut cache = HashMap::new();

        // Map our filetype names to syntect's syntax names
        cache.insert("Rust".to_string(), "Rust".to_string());
        cache.insert("Python".to_string(), "Python".to_string());
        cache.insert("JavaScript".to_string(), "JavaScript".to_string());
        cache.insert(
            "JavaScript (React)".to_string(),
            "JavaScript (JSX)".to_string(),
        );
        cache.insert("TypeScript".to_string(), "TypeScript".to_string());
        cache.insert(
            "TypeScript (React)".to_string(),
            "TypeScriptReact".to_string(),
        );
        cache.insert("C".to_string(), "C".to_string());
        cache.insert("C++".to_string(), "C++".to_string());
        cache.insert("Java".to_string(), "Java".to_string());
        cache.insert("Go".to_string(), "Go".to_string());
        cache.insert("Ruby".to_string(), "Ruby".to_string());
        cache.insert("PHP".to_string(), "PHP".to_string());
        cache.insert("Swift".to_string(), "Swift".to_string());
        cache.insert("Kotlin".to_string(), "Kotlin".to_string());
        cache.insert("Scala".to_string(), "Scala".to_string());
        cache.insert("C#".to_string(), "C#".to_string());
        cache.insert("F#".to_string(), "F#".to_string());
        cache.insert("Haskell".to_string(), "Haskell".to_string());
        cache.insert("OCaml".to_string(), "OCaml".to_string());
        cache.insert("Elixir".to_string(), "Elixir".to_string());
        cache.insert("Erlang".to_string(), "Erlang".to_string());
        cache.insert("Clojure".to_string(), "Clojure".to_string());
        cache.insert("Lua".to_string(), "Lua".to_string());
        cache.insert("Perl".to_string(), "Perl".to_string());
        cache.insert("R".to_string(), "R".to_string());
        cache.insert("Shell".to_string(), "Bourne Again Shell (bash)".to_string());
        cache.insert("PowerShell".to_string(), "PowerShell".to_string());
        cache.insert("SQL".to_string(), "SQL".to_string());
        cache.insert("HTML".to_string(), "HTML".to_string());
        cache.insert("CSS".to_string(), "CSS".to_string());
        cache.insert("SCSS".to_string(), "SCSS".to_string());
        cache.insert("Less".to_string(), "Less".to_string());
        cache.insert("JSON".to_string(), "JSON".to_string());
        cache.insert("YAML".to_string(), "YAML".to_string());
        cache.insert("TOML".to_string(), "TOML".to_string());
        cache.insert("XML".to_string(), "XML".to_string());
        cache.insert("Markdown".to_string(), "Markdown".to_string());
        cache.insert(
            "reStructuredText".to_string(),
            "reStructuredText".to_string(),
        );
        cache.insert("LaTeX".to_string(), "LaTeX".to_string());
        cache.insert("Makefile".to_string(), "Makefile".to_string());

        cache
    }

    /// Get available theme names
    pub fn available_themes(&self) -> Vec<&str> {
        self.theme_set.themes.keys().map(|s| s.as_str()).collect()
    }

    /// Set the current theme by name
    pub fn set_theme(&mut self, name: &str) -> bool {
        if self.theme_set.themes.contains_key(name) {
            self.current_theme = name.to_string();
            true
        } else {
            false
        }
    }

    /// Get the current theme
    pub fn current_theme(&self) -> Option<&Theme> {
        self.theme_set.themes.get(&self.current_theme)
    }

    /// Find a syntax by our filetype name
    pub fn syntax_for_filetype(&self, filetype: &str) -> Option<&SyntaxReference> {
        // First, try our cache mapping
        if let Some(syntax_name) = self.filetype_cache.get(filetype) {
            if let Some(syntax) = self.syntax_set.find_syntax_by_name(syntax_name) {
                return Some(syntax);
            }
        }

        // Fall back to trying the filetype name directly
        if let Some(syntax) = self.syntax_set.find_syntax_by_name(filetype) {
            return Some(syntax);
        }

        None
    }

    /// Get the plain text syntax (fallback)
    pub fn plain_text_syntax(&self) -> &SyntaxReference {
        self.syntax_set.find_syntax_plain_text()
    }
}

impl Default for HighlightingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A styled span of text for rendering
#[derive(Debug, Clone, Default)]
pub struct StyledSpan {
    pub text: String,
    pub style: HighlightStyle,
}

/// Simplified highlight style for ratatui conversion
#[derive(Debug, Clone, Copy, Default)]
pub struct HighlightStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

/// Per-line highlighting cache for a document
#[derive(Debug)]
pub struct LineHighlightCache {
    /// Cached highlighted spans per line
    lines: Vec<Option<Vec<StyledSpan>>>,
    /// Dirty flags per line (true = needs re-highlighting)
    dirty: Vec<bool>,
    /// The syntax being used (name, for invalidation)
    syntax_name: String,
    /// The theme being used (for invalidation)
    theme_name: String,
}

impl LineHighlightCache {
    pub fn new(line_count: usize, syntax_name: &str, theme_name: &str) -> Self {
        Self {
            lines: vec![None; line_count],
            dirty: vec![true; line_count],
            syntax_name: syntax_name.to_string(),
            theme_name: theme_name.to_string(),
        }
    }

    /// Mark a line as dirty (needs re-highlighting)
    pub fn mark_dirty(&mut self, line: usize) {
        if line < self.dirty.len() {
            self.dirty[line] = true;
            // Also mark subsequent lines as dirty since highlighting can depend on previous lines
            for i in (line + 1)..self.dirty.len() {
                self.dirty[i] = true;
            }
        }
    }

    /// Mark all lines as dirty
    pub fn mark_all_dirty(&mut self) {
        for d in &mut self.dirty {
            *d = true;
        }
    }

    /// Check if theme/syntax changed and invalidate if needed
    pub fn check_invalidation(&mut self, syntax_name: &str, theme_name: &str) {
        if self.syntax_name != syntax_name || self.theme_name != theme_name {
            self.syntax_name = syntax_name.to_string();
            self.theme_name = theme_name.to_string();
            self.mark_all_dirty();
        }
    }

    /// Resize the cache for a new line count
    pub fn resize(&mut self, new_line_count: usize) {
        self.lines.resize(new_line_count, None);
        self.dirty.resize(new_line_count, true);
    }

    /// Get cached line if available and not dirty
    pub fn get(&self, line: usize) -> Option<&Vec<StyledSpan>> {
        if line < self.lines.len() && !self.dirty[line] {
            self.lines[line].as_ref()
        } else {
            None
        }
    }

    /// Store highlighted line
    pub fn set(&mut self, line: usize, spans: Vec<StyledSpan>) {
        if line < self.lines.len() {
            self.lines[line] = Some(spans);
            self.dirty[line] = false;
        }
    }
}
