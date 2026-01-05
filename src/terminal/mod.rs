use anyhow::Result;
use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use std::io::{Read, Write};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;

/// Terminal emulator state
pub struct Terminal {
    /// The PTY pair (master/slave)
    pty_pair: PtyPair,
    /// PTY writer for sending input
    writer: Box<dyn Write + Send>,
    /// VT100 parser for interpreting terminal output
    parser: Arc<Mutex<vt100::Parser>>,
    /// Receiver for data from the reader thread
    rx: Receiver<Vec<u8>>,
    /// Current terminal size
    pub cols: u16,
    pub rows: u16,
    /// Scroll offset (0 = at bottom/current, positive = scrolled up)
    pub scroll_offset: usize,
}

impl Terminal {
    /// Create a new terminal with the given size
    pub fn new(cols: u16, rows: u16) -> Result<Self> {
        let pty_system = native_pty_system();

        let pty_pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Get the shell from environment
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

        // Spawn the shell
        let mut cmd = CommandBuilder::new(&shell);
        cmd.cwd(std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/")));

        // Set some environment variables
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");

        let _child = pty_pair.slave.spawn_command(cmd)?;

        // Get writer for sending input to the PTY
        let writer = pty_pair.master.take_writer()?;

        // Create VT100 parser
        let parser = Arc::new(Mutex::new(vt100::Parser::new(rows, cols, 1000)));

        // Create channel for PTY output
        let (tx, rx) = channel();

        // Spawn reader thread
        let mut reader = pty_pair.master.try_clone_reader()?;
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        // EOF - shell exited
                        break;
                    }
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        });

        Ok(Self {
            pty_pair,
            writer,
            parser,
            rx,
            cols,
            rows,
            scroll_offset: 0,
        })
    }

    /// Read available output from the PTY and process it
    pub fn read_output(&mut self) -> Result<()> {
        // Read all available data from the channel (non-blocking)
        loop {
            match self.rx.try_recv() {
                Ok(data) => {
                    let mut parser = self.parser.lock().unwrap();
                    parser.process(&data);
                }
                Err(TryRecvError::Empty) => {
                    // No more data available
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    // Reader thread exited
                    break;
                }
            }
        }

        Ok(())
    }

    /// Send input to the terminal
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.writer.write_all(data)?;
        self.writer.flush()?;
        Ok(())
    }

    /// Send a character to the terminal
    pub fn send_char(&mut self, c: char) -> Result<()> {
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        self.write(s.as_bytes())
    }

    /// Send a key sequence to the terminal
    pub fn send_key(
        &mut self,
        key: crossterm::event::KeyCode,
        modifiers: crossterm::event::KeyModifiers,
    ) -> Result<()> {
        use crossterm::event::{KeyCode, KeyModifiers};

        let ctrl = modifiers.contains(KeyModifiers::CONTROL);
        let alt = modifiers.contains(KeyModifiers::ALT);

        let seq: Option<&[u8]> = match key {
            KeyCode::Enter => Some(b"\r"),
            KeyCode::Backspace => Some(b"\x7f"),
            KeyCode::Tab => Some(b"\t"),
            KeyCode::Esc => Some(b"\x1b"),
            KeyCode::Up => Some(b"\x1b[A"),
            KeyCode::Down => Some(b"\x1b[B"),
            KeyCode::Right => Some(b"\x1b[C"),
            KeyCode::Left => Some(b"\x1b[D"),
            KeyCode::Home => Some(b"\x1b[H"),
            KeyCode::End => Some(b"\x1b[F"),
            KeyCode::PageUp => Some(b"\x1b[5~"),
            KeyCode::PageDown => Some(b"\x1b[6~"),
            KeyCode::Delete => Some(b"\x1b[3~"),
            KeyCode::Insert => Some(b"\x1b[2~"),
            KeyCode::F(1) => Some(b"\x1bOP"),
            KeyCode::F(2) => Some(b"\x1bOQ"),
            KeyCode::F(3) => Some(b"\x1bOR"),
            KeyCode::F(4) => Some(b"\x1bOS"),
            KeyCode::F(5) => Some(b"\x1b[15~"),
            KeyCode::F(6) => Some(b"\x1b[17~"),
            KeyCode::F(7) => Some(b"\x1b[18~"),
            KeyCode::F(8) => Some(b"\x1b[19~"),
            KeyCode::F(9) => Some(b"\x1b[20~"),
            KeyCode::F(10) => Some(b"\x1b[21~"),
            KeyCode::F(11) => Some(b"\x1b[23~"),
            KeyCode::F(12) => Some(b"\x1b[24~"),
            KeyCode::Char(c) => {
                if ctrl {
                    // Ctrl+letter sends control character (ASCII 1-26)
                    if c.is_ascii_lowercase() || c.is_ascii_uppercase() {
                        let ctrl_char = (c.to_ascii_lowercase() as u8) - b'a' + 1;
                        return self.write(&[ctrl_char]);
                    }
                }
                if alt {
                    // Alt+char sends ESC followed by the char
                    let mut buf = [0u8; 5];
                    buf[0] = 0x1b;
                    let len = c.encode_utf8(&mut buf[1..]).len();
                    return self.write(&buf[..1 + len]);
                }
                // Regular character
                return self.send_char(c);
            }
            _ => None,
        };

        if let Some(seq) = seq {
            self.write(seq)?;
        }

        Ok(())
    }

    /// Resize the terminal
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.cols = cols;
        self.rows = rows;

        self.pty_pair.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut parser = self.parser.lock().unwrap();
        parser.set_size(rows, cols);

        Ok(())
    }

    /// Get the current screen contents
    pub fn screen(&self) -> std::sync::MutexGuard<'_, vt100::Parser> {
        self.parser.lock().unwrap()
    }

    /// Scroll up in the scrollback buffer
    pub fn scroll_up(&mut self, lines: usize) {
        let mut parser = self.parser.lock().unwrap();
        let current = parser.screen().scrollback();
        let new_offset = current + lines;
        parser.set_scrollback(new_offset);
        // Update our tracking (parser clamps to actual available)
        self.scroll_offset = parser.screen().scrollback();
    }

    /// Scroll down in the scrollback buffer
    pub fn scroll_down(&mut self, lines: usize) {
        let mut parser = self.parser.lock().unwrap();
        let current = parser.screen().scrollback();
        let new_offset = current.saturating_sub(lines);
        parser.set_scrollback(new_offset);
        self.scroll_offset = new_offset;
    }

    /// Scroll to the bottom (most recent output)
    pub fn scroll_to_bottom(&mut self) {
        let mut parser = self.parser.lock().unwrap();
        parser.set_scrollback(0);
        self.scroll_offset = 0;
    }

    /// Check if we're scrolled back (not at the bottom)
    pub fn is_scrolled_back(&self) -> bool {
        let parser = self.parser.lock().unwrap();
        parser.screen().scrollback() > 0
    }

    /// Get the maximum scrollback size (configured limit, not current content)
    pub fn max_scrollback(&self) -> usize {
        // We configured 1000 lines of scrollback
        1000
    }

    /// Get the current scrollback offset
    pub fn current_scrollback(&self) -> usize {
        let parser = self.parser.lock().unwrap();
        parser.screen().scrollback()
    }
}
