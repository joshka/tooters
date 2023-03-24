use std::io::{self, Stdout};

use crossterm::{
    event::{EventStream, KeyCode, KeyModifiers},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{backend::CrosstermBackend, Frame, Terminal};
use tokio::sync::mpsc;

use crate::app::Event;

type Backend = CrosstermBackend<Stdout>;

pub struct Tui {
    terminal: Terminal<Backend>,
}

impl Tui {
    pub fn build() -> crate::Result<Self> {
        let stdout = io::stdout();
        let backend = Backend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn init(&mut self) -> crate::Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    pub async fn handle_key_events(tx: mpsc::Sender<Event>) {
        let mut key_events = EventStream::new();
        while let Some(Ok(crossterm::event::Event::Key(key_event))) = key_events.next().await {
            match (key_event.modifiers, key_event.code) {
                (crossterm::event::KeyModifiers::CONTROL, KeyCode::Char('c'))
                | (KeyModifiers::NONE, KeyCode::Char('q')) => {
                    tx.send(Event::Quit).await.unwrap();
                }
                _ => {
                    tx.send(Event::Key(key_event)).await.unwrap();
                }
            }
        }
    }

    // pub fn draw<F>(&mut self, f: F) -> crate::Result<()>
    // where
    //     F: FnOnce(&mut Frame<Backend>),
    // {
    //     let t = &mut self.terminal;
    //     t.draw(f)?;
    //     Ok(())
    // }
}

impl Drop for Tui {
    fn drop(&mut self) {
        terminal::disable_raw_mode().unwrap();
        crossterm::execute!(io::stdout(), LeaveAlternateScreen).unwrap();
        self.terminal.show_cursor().unwrap();
    }
}
