use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, EventStream};
use futures::StreamExt;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::time::Duration;
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    task::JoinHandle,
    time::interval,
};
use tracing::{error, info, trace};

const TICK_RATE: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Tick,
    Quit,
    CrosstermEvent(CrosstermEvent),
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

    pub fn start(&self) -> Result<()> {
        info!("Starting event loop");
        self.spawn_tick_task();
        self.spawn_signal_task()?;
        self.spawn_crossterm_task();
        Ok(())
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    /// Sends a tick event every tick_rate
    fn spawn_tick_task(&self) -> JoinHandle<()> {
        let tx = self.tx.clone();
        let mut interval = interval(TICK_RATE);
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                trace!("tick");
                if tx.send(Event::Tick).await.is_err() {
                    error!("Failed to send tick event");
                    break;
                }
            }
        })
    }

    /// Handle signals so killing the process cleans up the terminal correctly
    fn spawn_signal_task(&self) -> Result<JoinHandle<()>> {
        let tx = self.tx.clone();
        let mut signals = Signals::new([SIGHUP, SIGTERM, SIGINT, SIGQUIT])?;
        Ok(tokio::spawn(async move {
            while let Some(signal) = signals.next().await {
                match signal {
                    SIGTERM | SIGINT | SIGQUIT => {
                        info!("Received signal {}, shutting down", signal);
                        // Shutdown the system;
                        if tx.send(Event::Quit).await.is_err() {
                            error!("Failed to send quit event");
                            break;
                        }
                    }
                    _ => {
                        error!("Received unexpected signal {}", signal);
                    }
                }
            }
        }))
    }

    /// Sends crossterm events to the event channel
    fn spawn_crossterm_task(&self) -> JoinHandle<()> {
        let tx = self.tx.clone();
        let mut events = EventStream::new();
        tokio::spawn(async move {
            while let Some(Ok(event)) = events.next().await {
                trace!(crossterm_event = ?event);
                if tx.send(Event::CrosstermEvent(event)).await.is_err() {
                    error!("Failed to send event to channel");
                    break;
                }
            }
        })
    }
}
