mod home;
mod login;

use std::fmt::Display;
use tokio::sync::mpsc;

use crate::app::Event;
use home::HomeView;
use login::LoginView;

pub enum View {
    Login(LoginView),
    Home(HomeView),
    None,
}

impl Display for View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            View::Login(view) => write!(f, "{}", view),
            View::Home(view) => write!(f, "{}", view),
            View::None => write!(f, "None"),
        }
    }
}

impl View {
    pub fn login() -> Self {
        Self::Login(LoginView::new())
    }

    pub fn home() -> Self {
        Self::Home(HomeView::new())
    }

    pub async fn run(self, event_tx: mpsc::Sender<Event>) {
        tokio::spawn(async move {
            let _result = match self {
                View::Login(view) => view.run(event_tx).await,
                View::Home(view) => view.run(event_tx).await,
                View::None => Ok(()),
            };
        });
    }
}
