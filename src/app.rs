use crate::{
    event::{Event, Events, Outcome},
    logging::LogMessage,
    root::Root,
    ui::UI,
};
use anyhow::{Context, Result};
use crossterm::event::{
    Event::Key,
    KeyCode::{self, Char},
};
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::{debug, error, info, trace};

/// Runs the application.
/// This function is the entry point of the application.
///
/// # Errors
/// Will return an error if the application fails to initialize.
/// Will return an error if the application fails to run.
pub async fn run(logs: Arc<Mutex<Vec<LogMessage>>>) -> Result<()> {
    let mut app = App::new(logs)?;
    app.run().await?;
    Ok(())
}

struct App {
    events: Events,
    ui: UI,
    root: Root,
}

impl App {
    pub fn new(logs: Arc<Mutex<Vec<LogMessage>>>) -> Result<Self> {
        let events = Events::new();
        let root = Root::new(events.tx.clone(), logs);
        let ui = UI::new().context("Initializing UI failed")?;
        Ok(Self { events, ui, root })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting application");
        self.ui.start().context("Start UI failed")?;
        self.events.start().context("Starting events failed")?;
        self.root
            .start()
            .await
            .context("Starting root component failed")?;
        self.main_loop().await.context("Running main loop failed")?;
        info!("Shutting down");
        Ok(())
    }

    async fn main_loop(&mut self) -> Result<(), anyhow::Error> {
        loop {
            self.ui.draw(|f| {
                self.root.draw(f, f.size());
            })?;
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
                    if self.root.handle_event(&event).await == Outcome::Consumed {
                        debug!(?event, "Event consumed by root component");
                        continue;
                    }
                    if let Event::CrosstermEvent(Key(key)) = event {
                        if key.code == Char('q') || key.code == KeyCode::Esc {
                            debug!("Received quit key");
                            break;
                        }
                    }
                }
                _ => {
                    error!("Event channel closed. Exiting as we won't receive any more events.");
                    break;
                }
            }
        }
        Ok(())
    }
}
