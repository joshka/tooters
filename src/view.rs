mod home;
pub(crate) mod login;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
};
use std::fmt::Display;
use tokio::sync::mpsc;

use crate::app::Event;
use home::HomeView;
use login::LoginView;

use self::login::LoginDetails;

#[derive(Debug, Clone)]
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

    pub fn home(login_details: LoginDetails) -> Self {
        Self::Home(HomeView::new(
            login_details.account.username,
            login_details.url,
            login_details.mastodon_client,
        ))
    }

    pub async fn run(self, event_tx: mpsc::Sender<Event>) {
        tokio::spawn(async move {
            let _result = match self {
                View::Login(view) => view.run(event_tx).await,
                View::Home(mut view) => view.run(event_tx).await,
                View::None => Ok(()),
            };
        });
    }

    pub fn widget(&self) -> Box<dyn Widget> {
        match self {
            View::Login(view) => Box::new(view.widget()),
            View::Home(view) => Box::new(view.widget()),
            View::None => Box::new(Paragraph::new("None")),
        }
    }
}

// impl From<View> for dyn Widget {
//     fn from(view: View) -> Self {
//         match view {
//             View::Login(view) => view.widget().into(),
//             View::Home(view) => view.widget().into(),
//             View::None => Paragraph::new("None").into(),
//         }
//     }
// }

impl Widget for View {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            View::Login(view) => view.widget().render(area, buf),
            View::Home(view) => view.widget().render(area, buf),
            View::None => Paragraph::new("None").render(area, buf),
        }
    }
}
