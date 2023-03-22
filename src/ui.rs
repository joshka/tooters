use std::io::{self, Stdout};

use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::{backend::CrosstermBackend, Frame, Terminal};

use crate::app::AppResult;

type Backend = CrosstermBackend<Stdout>;

pub struct Ui {
    terminal: Terminal<Backend>,
}

impl Ui {
    pub fn new() -> AppResult<Self> {
        let stdout = io::stdout();
        let backend = Backend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn init(&mut self) -> AppResult<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    pub fn draw<F>(&mut self, f: F) -> AppResult<()>
    where
        F: FnOnce(&mut Frame<Backend>),
    {
        self.terminal.draw(f)?;
        Ok(())
    }
}

impl Drop for Ui {
    fn drop(&mut self) {
        terminal::disable_raw_mode().unwrap();
        crossterm::execute!(io::stdout(), LeaveAlternateScreen).unwrap();
        self.terminal.show_cursor().unwrap();
    }
}
