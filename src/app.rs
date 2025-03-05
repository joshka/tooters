use color_eyre::{eyre::WrapErr, Result};
use crossterm::event::{Event::Key, KeyCode::Char};
use ratatui::DefaultTerminal;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace};

use crate::{
    event::{Event, Events, Outcome},
    logging::LogCollector,
    root::Root,
};

pub struct App {
    shutdown: CancellationToken,
    events: Events,
    root: Root,
}

impl App {
    pub fn new(logs: LogCollector) -> Self {
        let events = Events::new();
        let root = Root::new(events.tx.clone(), logs);
        let shutdown = CancellationToken::new();
        Self {
            events,
            root,
            shutdown,
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        info!("Starting application");
        self.events.start();
        self.root.start();
        self.main_loop(&mut terminal)
            .await
            .wrap_err("Running main loop failed")?;
        info!("Shutting down");
        Ok(())
    }

    async fn main_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.shutdown.is_cancelled() {
            self.draw(terminal)?;
            self.handle_events().await;
        }
        Ok(())
    }

    fn draw(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        terminal
            .draw(|frame| frame.render_widget(&self.root, frame.area()))
            .wrap_err("failed to draw")?;
        Ok(())
    }

    async fn handle_events(&mut self) {
        let Some(event) = self.events.next().await else {
            error!("event channel closed");
            self.shutdown.cancel();
            return;
        };
        match event {
            Event::Quit => {
                info!("received Quit event");
                self.shutdown.cancel();
            }
            Event::Tick => {
                trace!("received Tick event");
                self.root.handle_event(&Event::Tick).await;
            }
            _ => {
                if self.root.handle_event(&event).await == Outcome::Handled {
                    debug!(?event, "event handled by root component");
                } else if let Event::Crossterm(Key(key)) = event {
                    if key.code == Char('q') {
                        debug!("received quit key");
                        self.shutdown.cancel();
                    }
                }
            }
        }
    }
}
