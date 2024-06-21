use crate::{
    authentication::Authentication,
    event::{Event, Outcome},
    home::Home,
    logging::LogCollector,
    widgets::{StatusBar, TitleBar},
};

use color_eyre::{eyre::WrapErr, Result};
use parking_lot::RwLock;
use ratatui::prelude::*;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tracing::info;

enum State {
    Authentication,
    Home,
}

pub struct Root {
    _event_sender: Sender<Event>,
    state: State,
    authentication: Authentication,
    home: Home,
    logs: LogCollector,
    show_logs: bool,
}

/// The root component is the top-level component of the application.
/// It is responsible for starting and stopping all other components.
/// It is also responsible for handling events and drawing the UI.
impl Root {
    pub fn new(event_sender: Sender<Event>, logs: LogCollector) -> Self {
        let authentication_data = Arc::new(RwLock::new(None));
        let authentication =
            Authentication::new(event_sender.clone(), Arc::clone(&authentication_data));
        let home = Home::new(event_sender.clone(), Arc::clone(&authentication_data));
        // show logs if we set TOOT_RS_SHOW_LOGS to anything
        let show_logs = std::env::var("TOOT_RS_SHOW_LOGS").is_ok();
        Self {
            _event_sender: event_sender,
            state: State::Authentication,
            authentication,
            home,
            logs,
            show_logs,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Starting root component");
        self.authentication
            .start()
            .await
            .wrap_err("Authentication component failed to start")
    }

    /// Handles an event.
    /// Returns an `Outcome` that indicates whether the event was handled or not.
    pub async fn handle_event(&mut self, event: &Event) -> Outcome {
        match self.state {
            State::Authentication => {
                if event == &Event::AuthenticationSuccess {
                    self.state = State::Home;
                    self.home.start().await.ok();
                    return Outcome::Handled;
                }
                self.authentication.handle_event(event).await
            }
            State::Home => self.home.handle_event(event),
        }
    }
}

impl Widget for &Root {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::*;
        let log_height = if self.show_logs { 7 } else { 0 };
        let [top, mid, logs, bottom] = Layout::vertical([
            Length(TitleBar::HEIGHT),
            Fill(1),
            Length(log_height),
            Length(StatusBar::HEIGHT),
        ])
        .areas(area);
        match self.state {
            State::Authentication => {
                TitleBar::new(&self.authentication.title()).render(top, buf);
                StatusBar::new("Loading...").render(bottom, buf);
                self.authentication.render(mid, buf);
            }
            State::Home => {
                TitleBar::new(self.home.title()).render(top, buf);
                StatusBar::new(self.home.status()).render(bottom, buf);
                self.home.render(mid, buf);
            }
        }
        if self.show_logs {
            self.logs.render(logs, buf);
        };
    }
}
