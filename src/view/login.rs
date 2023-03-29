use mastodon_async::{helpers::toml, Mastodon};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Widget},
};
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
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    pub async fn run(&self, event_tx: mpsc::Sender<Event>) {
        if let Some(login_details) = Self::load_credentials().await {
            if let Err(e) = event_tx.send(Event::LoggedIn(login_details)).await {
                eprintln!("Error sending login event: {}", e);
            }
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
}

impl Widget for LoginView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Logging in...")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.to_string()),
            )
            .render(area, buf);
    }
}
