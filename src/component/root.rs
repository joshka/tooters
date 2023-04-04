use super::{AuthenticationComponent, EventOutcome};

use crate::{
    event::Event,
    widgets::{StatusBar, TitleBar},
};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use tokio::sync::mpsc::Sender;
use tracing::info;

#[derive(Debug)]
pub struct RootComponent {
    _event_sender: Sender<Event>,
    auth: AuthenticationComponent,
}

impl RootComponent {
    pub fn new(_event_sender: Sender<Event>) -> Self {
        let auth = AuthenticationComponent::new(_event_sender.clone());
        Self {
            _event_sender,
            auth,
        }
    }

    pub async fn start(&mut self) {
        info!("Starting root component");
        self.auth.start().await.unwrap();
    }

    pub async fn handle_event(&mut self, event: &Event) -> EventOutcome {
        self.auth.handle_event(event).await
    }

    pub fn draw(&self, f: &mut Frame<impl Backend>, area: Rect) {
        if let [top, mid, bottom] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(TitleBar::HEIGHT),
                Constraint::Min(0),
                Constraint::Length(StatusBar::HEIGHT),
            ])
            .split(area)
        {
            f.render_widget(TitleBar::new(self.auth.title()), top);
            f.render_widget(StatusBar::new("Loading...".to_string()), bottom);
            self.auth.draw(f, mid);
        }
    }
}
