use std::path::Path;

use async_trait::async_trait;
use mastodon_async::{helpers::toml, Mastodon};
use tokio::sync::mpsc::Sender;

use crate::app::{Event, View};

pub struct IntitialView {
    status: String,
    tx: Sender<Event>,
}

impl IntitialView {
    pub fn new(tx: Sender<Event>) -> Self {
        Self { status: "Loading...".to_string(), tx }
    }

    async fn attempt_login(&mut self, p: &Path) {
        let tx = self.tx.clone();
        match toml::from_file(p) {
            Ok(credentials) => {
                let server_name = &credentials.base;
                self.status = format!("Logging in to {server_name}...");
                let mastodon = Mastodon::from(credentials.clone());
                match mastodon.verify_credentials().await {
                    Ok(account) => {
                        // TODO: also return the mastodon instance for future use
                        let username = &account.username;
                        self.status = format!("Logged in as {username}@{server_name}");
                         tx.send(Event::LoggedIn(account.username)).await;
                    }
                    Err(e) => match e {
                        mastodon_async::Error::Serde(_) => {
                            let msg = "Login failed: Could not understand the server's response";
                            self.tx.send(Event::LoggedOut(msg.to_string())).await;
                        }
                        mastodon_async::Error::Api { status, response } => {
                            let msg = format!("Login failed: {} {}", status, response.error);
                            self.tx.send(Event::LoggedOut(msg.to_string())).await;
                        }
                        _ => {
                            let msg = format!("Login failed: {}", e);
                            self.tx.send(Event::LoggedOut(msg.to_string())).await;
                        }
                    },
                }
            }
            Err(_) => {
                let msg = "No previous login".to_string();
                self.tx.send(Event::LoggedOut(msg)).await;
            }
        }
    }
}

#[async_trait]
impl View for IntitialView {
    fn title(&self) -> String {
        self.status.clone()
    }

    async fn run(&mut self) {
        IntitialView::attempt_login(self, Path::new("mastodon-data.toml")).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mastodon_async::helpers::toml::to_file;
    use mastodon_async::Data;
    use tempfile::tempdir;
    use tokio::sync::mpsc::channel;

    #[tokio::test]
    async fn test_attempt_login() {
        let (tx, mut rx) = channel(100);
        let mut view = IntitialView::new(tx);
        view.attempt_login(Path::new("")).await;
        assert_eq!(
            rx.recv().await,
            Some(Event::LoggedOut("No previous login".to_string()))
        );
    }

    #[tokio::test]
    async fn test_attempt_login_with_bad_credentials() {
        // femme::with_level(log::LevelFilter::Trace);

        let data = Data {
            base: "https://mastodon.social".into(),
            client_id: "adbc01234".into(),
            client_secret: "0987dcba".into(),
            redirect: "urn:ietf:wg:oauth:2.0:oob".into(),
            token: "fedc5678".into(),
        };
        let tempdir = tempdir().expect("Couldn't create tempdir");
        let filename = tempdir.path().join("mastodon-data.toml");
        to_file(&data, &filename).expect("Couldn't write to file");

        let (tx, mut rx) = channel(100);
        let mut view = IntitialView::new(tx);
        view.attempt_login(&filename).await;
        assert_eq!(
            rx.recv().await,
            Some(Event::LoggedOut(
                "Login failed: 401 Unauthorized The access token is invalid".to_string()
            ))
        );
    }

    #[tokio::test]
    #[ignore]
    async fn test_attempt_login_with_real_credentials() {
        let (tx, mut rx) = channel(100);
        let mut view = IntitialView::new(tx);
        view.attempt_login(Path::new("mastodon-data.toml")).await;
        assert_eq!(
            rx.recv().await,
            Some(Event::LoggedIn("joshka".to_string()))
        );
    }
}
