mod login;
use std::fmt::Display;

pub use login::LoginView;

mod home;
pub use home::HomeView;
use tokio::sync::mpsc;

use crate::app::Event;

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
    pub fn login(tx: mpsc::Sender<Event>) -> Self {
        Self::Login(LoginView::new(tx))
    }

    pub fn home(tx: mpsc::Sender<Event>) -> Self {
        Self::Home(HomeView::new(tx))
    }

    pub async fn run(self) {
        tokio::spawn(async move {
            let _result = match self {
                View::Login(view) => view.run().await,
                View::Home(view) => view.run().await,
                View::None => Ok(()),
            };
        });
    }
}