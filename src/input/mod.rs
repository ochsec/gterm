pub mod event;
pub mod keyboard;
pub mod mouse;

pub use event::AppEvent;

/// Handles input events and translates them to application commands
pub struct InputHandler {
    /// Tracks double/triple click timing for word/line selection
    last_click_time: std::time::Instant,
    last_click_pos: (u16, u16),
    click_count: u8,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            last_click_time: std::time::Instant::now(),
            last_click_pos: (0, 0),
            click_count: 0,
        }
    }

    /// Record a click and return the click count (1 = single, 2 = double, 3 = triple)
    pub fn record_click(&mut self, x: u16, y: u16) -> u8 {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_click_time);

        // Reset if too much time has passed or position changed
        if elapsed.as_millis() > 500 || (x, y) != self.last_click_pos {
            self.click_count = 1;
        } else {
            self.click_count = (self.click_count % 3) + 1;
        }

        self.last_click_time = now;
        self.last_click_pos = (x, y);
        self.click_count
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}
