use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub editor: EditorConfig,
    pub terminal: TerminalConfig,
    pub ui: UiConfig,
    pub file_tree: FileTreeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    /// Tab width in spaces
    #[serde(default = "default_tab_width")]
    pub tab_width: usize,
    /// Use spaces instead of tabs
    #[serde(default = "default_true")]
    pub insert_spaces: bool,
    /// Enable auto-indentation
    #[serde(default = "default_true")]
    pub auto_indent: bool,
    /// Show line numbers
    #[serde(default = "default_true")]
    pub show_line_numbers: bool,
    /// Highlight current line
    #[serde(default = "default_true")]
    pub highlight_current_line: bool,
    /// Word wrap
    #[serde(default)]
    pub word_wrap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    /// Custom shell command (empty = use $SHELL)
    #[serde(default)]
    pub shell: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Show sidebar
    #[serde(default = "default_true")]
    pub show_sidebar: bool,
    /// Show terminal
    #[serde(default = "default_true")]
    pub show_terminal: bool,
    /// Sidebar width percentage
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: u16,
    /// Terminal height percentage
    #[serde(default = "default_terminal_height")]
    pub terminal_height: u16,
    /// Theme name
    #[serde(default = "default_theme")]
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTreeConfig {
    /// Show hidden files
    #[serde(default)]
    pub show_hidden: bool,
    /// Follow current file in tree
    #[serde(default = "default_true")]
    pub follow_current_file: bool,
}

// Default value helpers
fn default_tab_width() -> usize {
    4
}
fn default_true() -> bool {
    true
}
fn default_sidebar_width() -> u16 {
    20
}
fn default_terminal_height() -> u16 {
    30
}
fn default_theme() -> String {
    "dark".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig::default(),
            terminal: TerminalConfig::default(),
            ui: UiConfig::default(),
            file_tree: FileTreeConfig::default(),
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_width: 4,
            insert_spaces: true,
            auto_indent: true,
            show_line_numbers: true,
            highlight_current_line: true,
            word_wrap: false,
        }
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            shell: String::new(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_sidebar: true,
            show_terminal: true,
            sidebar_width: 20,
            terminal_height: 30,
            theme: "dark".to_string(),
        }
    }
}

impl Default for FileTreeConfig {
    fn default() -> Self {
        Self {
            show_hidden: false,
            follow_current_file: true,
        }
    }
}

impl Config {
    /// Load configuration from the default config file location
    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(config) = toml::from_str(&content) {
                        return config;
                    }
                }
            }
        }
        Self::default()
    }

    /// Get the path to the config file
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("gterm").join("config.toml"))
    }

    /// Save configuration to the default config file location
    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let content = toml::to_string_pretty(self)?;
            std::fs::write(path, content)?;
        }
        Ok(())
    }
}
