use mastodon_async::{helpers::toml, Mastodon};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::fmt::Display;
use tokio::sync::mpsc;

use crate::{Event, LoginDetails};

#[derive(Debug, Default, Clone)]
pub struct LoginView;

impl Display for LoginView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Login")
    }
}

impl LoginView {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&self, event_tx: mpsc::Sender<Event>) {
        if let Some(login_details) = Self::load_credentials().await {
            event_tx
                .send(Event::LoggedIn(login_details))
                .await
                .expect("Failed to send login event");
        }
    }

    pub fn widget(&self) -> Paragraph<'static> {
        Paragraph::new("Logging in...").block(
            Block::default()
                .borders(Borders::ALL)
                .title(self.to_string()),
        )
    }

    async fn load_credentials() -> Option<LoginDetails> {
        match toml::from_file("mastodon-data.toml") {
            Ok(data) => {
                let mastodon = Mastodon::from(data.clone());
                match mastodon.verify_credentials().await {
                    Ok(account) => Some(LoginDetails {
                        url: data.base.to_string(),
                        account,
                        mastodon_client: mastodon,
                    }),
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    }
}
