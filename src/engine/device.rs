use std::{
    io::{self, Write},
    ops::{Drop},
    error::Error
};

extern crate crossterm;
use crossterm::{
    cursor, style,
    terminal::{self, ClearType},
};

/// Provides rendering device.
/// To use device, valid terminal or console must be provided from OS.
pub struct Device {
    stdout: io::Stdout
}

impl Drop for Device {
    fn drop(&mut self) {
        // Clear screen and leave alternative screen.
        let _ = crossterm::execute!(
            self.stdout,    // stdout will be moved into closure.
            style::ResetColor,
            cursor::Show,
            terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

impl Device {
    /// Create new device instance. 
    pub fn new() -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        // Create new local stdout and setup alternative screen.
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, terminal::EnterAlternateScreen)?;
        crossterm::terminal::enable_raw_mode()?;

        // Set value into stdout of struct.
        Ok(Device { stdout })
    }

    /// Clear screen.
    pub fn clear(&mut self) -> Result<(), crossterm::ErrorKind> {
        // Clear screen.
        crossterm::execute!(&mut self.stdout,
            crossterm::style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Hide,
        )?;
        Ok(())
    }

    /// Move cursor to given `pos` and print given `string`.
    pub fn mv_print(&mut self, pos: (u8, u8), string: &str) -> Result<(), crossterm::ErrorKind> {
        let (x, y) = (pos.0 as u16, pos.1 as u16);
        crossterm::execute!(
            &mut self.stdout, 
            cursor::MoveTo(x, y), 
            style::Print(string)
        )?;
        Ok(())
    }
}
