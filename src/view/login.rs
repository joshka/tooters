use mastodon_async::{helpers::toml, prelude::Account, Mastodon};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::fmt::Display;
use tokio::sync::mpsc;

use crate::app::{AppResult, Event};

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

    pub async fn run(&self, event_tx: mpsc::Sender<Event>) -> AppResult<()> {
        if let Some(login_details) = load_credentials().await {
            event_tx.send(Event::LoggedIn(login_details)).await?;
        }
        Ok(())
    }

    pub(crate) fn widget(&self) -> Paragraph<'static> {
        Paragraph::new("Logging in...").block(
            Block::default()
                .borders(Borders::ALL)
                .title(self.to_string()),
        )
    }
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
#[derive(Debug)]
pub struct LoginDetails {
    pub url: String,
    pub account: Account,
    pub mastodon_client: mastodon_async::mastodon::Mastodon,
}
