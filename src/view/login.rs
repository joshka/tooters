use mastodon_async::{helpers::toml, Mastodon};
use ratatui::{
    backend::Backend,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::fmt::Display;
use tokio::sync::mpsc;

use crate::{Event, LoginDetails};

#[derive(Debug, Default)]
pub struct LoginView {
    status: String,
}

impl Display for LoginView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Login")
    }
}

impl LoginView {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn status(&self) -> String {
        self.status.clone()
    }

    pub async fn run(&mut self, event_tx: mpsc::Sender<Event>) {
        if let Some(login_details) = self.load_credentials().await {
            if let Err(e) = event_tx.send(Event::LoggedIn(login_details)).await {
                eprintln!("Error sending login event: {}", e);
            }
        }
    }

    async fn load_credentials(&mut self) -> Option<LoginDetails> {
        self.status = "Loading credentials...".to_string();
        match toml::from_file("mastodon-data.toml") {
            Ok(data) => {
                self.status = "Logging in...".to_string();
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

    pub fn draw(&self, frame: &mut Frame<impl Backend>, area: Rect) {
        let widget = Paragraph::new("Logging in...").block(
            Block::default()
                .borders(Borders::ALL)
                .title(self.to_string()),
        );
        frame.render_widget(widget, area);
    }
}
