use mastodon_async::{helpers::toml, Data, Mastodon};
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
    status: LoginStatus,
}

#[derive(Debug, Default)]
pub enum LoginStatus {
    #[default]
    LoadingCredentials,
    VerifyingCredentials,
    InvalidCredentials(String),
    FileError(String),
    LoggedIn,
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

    pub async fn run(&mut self, event_tx: mpsc::Sender<Event>) {
        match self.process_login().await {
            Ok(login_details) => {
                if let Err(e) = event_tx.send(Event::LoggedIn(login_details)).await {
                    eprintln!("Error sending login event: {}", e);
                }
            }
            Err(_e) => {}
        }
    }

    async fn process_login(&mut self) -> Result<LoginDetails, String> {
        self.status = LoginStatus::LoadingCredentials;
        match toml::from_file("mastodon-data.toml") {
            Ok(data) => self.verify_credentials(data).await,
            Err(e) => {
                self.status = LoginStatus::FileError(format!("File error: {}", e));
                Err(format!("File error: {}", e))
            }
        }
    }

    async fn verify_credentials(&mut self, data: Data) -> Result<LoginDetails, String> {
        self.status = LoginStatus::VerifyingCredentials;
        let mastodon = Mastodon::from(data.clone());

        match mastodon.verify_credentials().await {
            Ok(account) => {
                self.status = LoginStatus::LoggedIn;
                Ok(LoginDetails {
                    url: data.base.to_string(),
                    account,
                    mastodon_client: mastodon,
                })
            }
            Err(e) => {
                self.status = LoginStatus::InvalidCredentials(format!("Verification error: {}", e));
                Err(format!("Verification error: {}", e))
            }
        }
    }

    pub fn status(&self) -> String {
        match &self.status {
            LoginStatus::LoadingCredentials => "Loading...".to_string(),
            LoginStatus::VerifyingCredentials => "Verifying...".to_string(),
            LoginStatus::InvalidCredentials(_) => "Invalid credentials".to_string(),
            LoginStatus::FileError(_) => "File error".to_string(),
            LoginStatus::LoggedIn => "Logged in".to_string(),
        }
    }

    pub fn draw(&self, frame: &mut Frame<impl Backend>, area: Rect) {
        let message = match &self.status {
            LoginStatus::LoadingCredentials => "Loading credentials...",
            LoginStatus::VerifyingCredentials => "Verifying credentials...",
            LoginStatus::InvalidCredentials(msg) => msg,
            LoginStatus::FileError(msg) => msg,
            LoginStatus::LoggedIn => "Logged in",
        };

        let widget = Paragraph::new(message).block(
            Block::default()
                .borders(Borders::ALL)
                .title(self.to_string()),
        );
        frame.render_widget(widget, area);
    }
}
