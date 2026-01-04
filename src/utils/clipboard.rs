use anyhow::Result;

/// Clipboard operations
pub struct Clipboard {
    clipboard: Option<arboard::Clipboard>,
}

impl Clipboard {
    pub fn new() -> Self {
        let clipboard = arboard::Clipboard::new().ok();
        Self { clipboard }
    }

    /// Get text from clipboard
    pub fn get_text(&mut self) -> Result<String> {
        match &mut self.clipboard {
            Some(cb) => Ok(cb.get_text()?),
            None => anyhow::bail!("Clipboard not available"),
        }
    }

    /// Set text to clipboard
    pub fn set_text(&mut self, text: &str) -> Result<()> {
        match &mut self.clipboard {
            Some(cb) => {
                cb.set_text(text)?;
                Ok(())
            }
            None => anyhow::bail!("Clipboard not available"),
        }
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}
