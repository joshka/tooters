use crate::{
    authentication,
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

pub struct Component {
    _event_sender: Sender<Event>,
    auth: authentication::Component,
    logs: Arc<Mutex<Vec<LogMessage>>>,
}

impl Component {
    pub fn new(event_sender: Sender<Event>, logs: Arc<Mutex<Vec<LogMessage>>>) -> Self {
        let auth = authentication::Component::new(event_sender.clone());
        Self {
            _event_sender: event_sender,
            auth,
            logs,
        }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        info!("Starting root component");
        self.auth
            .start()
            .await
            .context("Failed to start authentication component")
    }

    pub async fn handle_event(&mut self, event: &Event) -> Outcome {
        self.auth.handle_event(event).await
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
            f.render_widget(TitleBar::new(&self.auth.title()), top);
            f.render_widget(StatusBar::new("Loading...".to_string()), bottom);
            f.render_widget(LogWidget::new(self.logs.clone()), logs);
            self.auth.draw(f, mid);
        }
    }
}
