use crate::{
    event::{Event, Events, Outcome},
    logging::LogCollector,
    root::Root,
};
use color_eyre::{eyre::WrapErr, Result};
use crossterm::event::{
    Event::Key,
    KeyCode::{self, Char},
};
use ratatui::backend::CrosstermBackend;
use std::io::Stdout;
use tracing::{debug, error, info, trace};

pub type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

pub struct App {
    events: Events,
    root: Root,
}

impl App {
    pub fn new(logs: LogCollector) -> Self {
        let events = Events::new();
        let root = Root::new(events.tx.clone(), logs);
        Self { events, root }
    }

    pub async fn run(mut self, mut terminal: Terminal) -> Result<()> {
        info!("Starting application");
        self.events.start().wrap_err("Starting events failed")?;
        self.root.start();
        self.main_loop(&mut terminal)
            .await
            .wrap_err("Running main loop failed")?;
        info!("Shutting down");
        Ok(())
    }

    async fn main_loop(&mut self, terminal: &mut Terminal) -> Result<()> {
        loop {
            self.draw(terminal)?;
            match self.events.next().await {
                Some(Event::Quit) => {
                    info!("Received quit event");
                    break;
                }
                Some(Event::Tick) => {
                    trace!("Received tick event");
                    self.root.handle_event(&Event::Tick).await;
                }
                Some(event) => {
                    if let Event::Crossterm(Key(key)) = event {
                        if key.code == KeyCode::Esc {
                            debug!("Received quit key");
                            break;
                        }
                    }
                    if self.root.handle_event(&event).await == Outcome::Handled {
                        debug!(?event, "Event handled by root component");
                        continue;
                    }
                    if let Event::Crossterm(Key(key)) = event {
                        if key.code == Char('q') {
                            debug!("Received quit key");
                            break;
                        }
                    }
                }
                None => {
                    error!("Event channel closed. Exiting as we won't receive any more events.");
                    break;
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, terminal: &mut Terminal) -> color_eyre::Result<()> {
        terminal
            .draw(|frame| frame.render_widget(&self.root, frame.area()))
            .wrap_err("failed to draw")?;
        Ok(())
    }
}
