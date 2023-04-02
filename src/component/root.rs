use super::{
    status_bar::StatusBar, title_bar::TitleBar, AuthenticationComponent, Component, EventOutcome,
};

use crate::event::Event;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use tokio::sync::mpsc::Sender;
use tracing::info;

#[derive(Debug)]
pub struct RootComponent {
    title: &'static str,
    _event_sender: Sender<Event>,
    auth: AuthenticationComponent,
}

impl RootComponent {
    pub fn new(_event_sender: Sender<Event>) -> Self {
        let auth = AuthenticationComponent::new(_event_sender.clone());
        Self {
            title: "Authentication",
            _event_sender,
            auth,
        }
    }
}

impl Component for RootComponent {
    fn draw(&self, f: &mut Frame<impl Backend>, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(TitleBar::HEIGHT),
                Constraint::Min(0),
                Constraint::Length(StatusBar::HEIGHT),
            ])
            .split(area);
        f.render_widget(TitleBar::new(self.title), layout[0]);
        self.auth.draw(f, layout[1]);
        f.render_widget(StatusBar::new("Loading...".to_string()), layout[2]);
    }

    fn handle_event(&mut self, event: &Event) -> EventOutcome {
        self.auth.handle_event(event)
    }

    fn start(&mut self) {
        info!("Starting root component");
        self.auth.start();
    }
}
