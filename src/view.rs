use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use std::fmt::Display;
use tokio::sync::mpsc;

use crate::{Event, LoginDetails};
use home::HomeView;
use login::LoginView;

pub mod home;
pub mod login;

#[derive(Debug, Clone)]
pub enum View {
    Login(LoginView),
    Home(HomeView),
    // None,
}

impl Display for View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Login(view) => write!(f, "{view}"),
            Self::Home(view) => write!(f, "{view}"),
        }
    }
}

impl View {
    #[must_use]
    pub fn login() -> Self {
        Self::Login(LoginView::new())
    }

    #[must_use]
    pub fn home(login_details: LoginDetails) -> Self {
        Self::Home(HomeView::from(login_details))
    }

    pub async fn run(self, event_tx: mpsc::Sender<Event>) {
        tokio::spawn(async move {
            match self {
                Self::Login(view) => view.run(event_tx).await,
                Self::Home(mut view) => view.run(event_tx).await,
            };
        });
    }
}

impl Widget for View {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            Self::Login(view) => view.widget().render(area, buf),
            Self::Home(view) => view.widget().render(area, buf),
        };
    }
}
