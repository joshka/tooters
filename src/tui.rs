use std::io::{self, Stdout};

use crossterm::{
    event::{EventStream, KeyCode, KeyModifiers},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{backend::CrosstermBackend, Frame, Terminal};
use tokio::sync::mpsc;

use crate::Event;

type Backend = CrosstermBackend<Stdout>;

pub struct Tui {
    terminal: Terminal<Backend>,
    tx: mpsc::Sender<Event>,
}

impl Tui {
    pub fn build(tx: mpsc::Sender<crate::Event>) -> crate::Result<Self> {
        Ok(Self {
            terminal: Terminal::new(Backend::new(io::stdout()))?,
            tx,
        })
    }

    pub async fn init(&mut self) -> crate::Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        self.start_keyboard_handler().await;
        Ok(())
    }

    async fn start_keyboard_handler(&self) {
        let tx = self.tx.clone();
        let mut key_events = EventStream::new();
        tokio::spawn(async move {
            while let Some(Ok(crossterm::event::Event::Key(key_event))) = key_events.next().await {
                match (key_event.modifiers, key_event.code) {
                    (crossterm::event::KeyModifiers::CONTROL, KeyCode::Char('c'))
                    | (KeyModifiers::NONE, KeyCode::Char('q')) => {
                        if tx.send(Event::Quit).await.is_err() {
                            break;
                        }
                    }
                    _ => {
                        if tx.send(Event::Key(key_event)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });
    }

    pub fn draw<F>(&mut self, f: F) -> crate::Result<()>
    where
        F: FnOnce(&mut Frame<Backend>),
    {
        let t = &mut self.terminal;
        t.draw(f)?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        terminal::disable_raw_mode().unwrap();
        crossterm::execute!(io::stdout(), LeaveAlternateScreen).unwrap();
        self.terminal.show_cursor().unwrap();
    }
}
