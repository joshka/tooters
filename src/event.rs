use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{Event as CrosstermEvent, EventStream};
use futures::StreamExt;
use signal_hook::consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use signal_hook_tokio::Signals;
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    time::interval,
};
use tracing::{error, info, trace};

/// The tick rate for the tick event (60fps)
const TICK_RATE: Duration = Duration::from_millis(1000 / 60);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Tick,
    Quit,
    Crossterm(CrosstermEvent),
    AuthenticationSuccess,
}

#[derive(Debug)]
pub struct Events {
    pub tx: Sender<Event>,
    rx: Receiver<Event>,
}

/// A wrapper around 3 event sources:
/// - Tick events
/// - Crossterm events
/// - Signals
impl Events {
    pub fn new() -> Self {
        let (tx, rx) = channel(100);
        Self { tx, rx }
    }

    pub fn start(&self) {
        info!("Starting event loop");
        tokio::spawn(Self::tick_task(self.tx.clone()));
        tokio::spawn(Self::signal_task(self.tx.clone()));
        tokio::spawn(Self::crossterm_task(self.tx.clone()));
    }

    /// Returns the next event
    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    /// Sends a tick event every `tick_rate`
    async fn tick_task(tx: Sender<Event>) {
        let mut interval = interval(TICK_RATE);
        loop {
            interval.tick().await;
            trace!("tick");
            if tx.send(Event::Tick).await.is_err() {
                break;
            }
        }
    }

    /// Handle signals so killing the process cleans up the terminal correctly
    async fn signal_task(tx: Sender<Event>) -> Result<()> {
        let mut signals = Signals::new([SIGHUP, SIGTERM, SIGINT, SIGQUIT])?;
        while let Some(signal) = signals.next().await {
            match signal {
                SIGTERM | SIGINT | SIGQUIT => {
                    info!("Received signal {}, shutting down", signal);
                    if tx.send(Event::Quit).await.is_err() {
                        break;
                    }
                }
                _ => {
                    error!("Received unexpected signal {}", signal);
                }
            }
        }
        Ok(())
    }

    /// Sends crossterm events to the event channel
    async fn crossterm_task(tx: Sender<Event>) {
        let mut events = EventStream::new();
        while let Some(Ok(event)) = events.next().await {
            trace!(crossterm_event = ?event);
            if tx.send(Event::Crossterm(event)).await.is_err() {
                break;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Handled,
    Ignored,
}
