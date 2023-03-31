use ratatui::{backend::Backend, layout::Rect, Frame};
use tokio::sync::mpsc;

use crate::Event;
use home::HomeView;
use login::LoginView;

pub mod home;
pub mod login;

#[derive(Debug)]
pub enum View {
    Login(LoginView),
    Home(HomeView),
    // None,
}

impl View {
    pub async fn run(&mut self, event_tx: mpsc::Sender<Event>) -> crate::Result<()> {
        match self {
            Self::Login(view) => view.run(event_tx).await,
            Self::Home(ref mut view) => view.run(event_tx).await,
        }
    }

    pub fn draw(&self, frame: &mut Frame<impl Backend>, area: Rect) {
        match self {
            Self::Login(view) => view.draw(frame, area),
            Self::Home(view) => view.draw(frame, area),
        }
    }

    pub fn title(&self) -> String {
        match self {
            Self::Login(view) => view.title(),
            Self::Home(view) => view.title(),
        }
    }

    pub fn status(&self) -> String {
        match self {
            Self::Login(view) => view.status(),
            Self::Home(view) => view.status(),
        }
    }
}
