use crate::{
    authentication::Authentication,
    event::{Event, Outcome},
    logging::{LogMessage, LogWidget},
    widgets::{StatusBar, TitleBar},
};
use anyhow::Context;
use parking_lot::Mutex;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tracing::info;

pub struct Root {
    _event_sender: Sender<Event>,
    authentication: Authentication,
    logs: Arc<Mutex<Vec<LogMessage>>>,
}

/// The root component is the top-level component of the application.
/// It is responsible for starting and stopping all other components.
/// It is also responsible for handling events and drawing the UI.
impl Root {
    pub fn new(event_sender: Sender<Event>, logs: Arc<Mutex<Vec<LogMessage>>>) -> Self {
        let authentication = Authentication::new(event_sender.clone());
        Self {
            _event_sender: event_sender,
            authentication,
            logs,
        }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        info!("Starting root component");
        self.authentication
            .start()
            .await
            .context("Authentication component failed to start")
    }

    /// Handles an event.
    /// Returns an `Outcome` that indicates whether the event was handled or not.
    pub async fn handle_event(&mut self, event: &Event) -> Outcome {
        self.authentication.handle_event(event).await
    }

    pub fn draw(&self, f: &mut Frame<impl Backend>, area: Rect) {
        if let [top, mid, logs, bottom] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(TitleBar::HEIGHT),
                Constraint::Min(0),
                Constraint::Length(20), // logs
                Constraint::Length(StatusBar::HEIGHT),
            ])
            .split(area)
        {
            f.render_widget(TitleBar::new(&self.authentication.title()), top);
            f.render_widget(StatusBar::new("Loading...".to_string()), bottom);
            f.render_widget(LogWidget::new(self.logs.clone()), logs);
            self.authentication.draw(f, mid);
        }
    }
}
