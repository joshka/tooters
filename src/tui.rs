use crossterm::terminal::{self, LeaveAlternateScreen};
use std::io;

pub fn restore() -> io::Result<()> {
    terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
