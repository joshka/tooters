use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::{self, Stdout},
    ops::{Deref, DerefMut},
};
use tracing::{error, info};

pub type Backend = CrosstermBackend<Stdout>;

pub struct Tui {
    terminal: Terminal<Backend>,
}

impl Tui {
    pub fn init() -> io::Result<Self> {
        info!("Starting UI");
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
        let mut terminal = Terminal::new(Backend::new(io::stdout()))?;
        terminal.hide_cursor()?;
        terminal.clear()?;
        Ok(Self { terminal })
    }
}

pub fn restore() -> io::Result<()> {
    crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

impl Drop for Tui {
    fn drop(&mut self) {
        info!("Shutting down UI");
        if let Err(e) = restore() {
            error!("Failed to restore terminal: {}", e);
        }
    }
}

impl Deref for Tui {
    type Target = Terminal<Backend>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}
