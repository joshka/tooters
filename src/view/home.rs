use std::{fmt::Display, time::Duration};

use mastodon_async::{mastodon::Mastodon, prelude::Status};
use ratatui::widgets::{Block, Borders, List, ListItem};
use tokio::{sync::mpsc, time::sleep};

use crate::{Event, LoginDetails};

#[derive(Debug, Clone)]
pub struct HomeView {
    username: String,
    url: String,
    mastodon_client: Mastodon,
    timeline: Option<Vec<Status>>,
}

impl Display for HomeView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}@{}",
            self.username,
            self.url.trim_start_matches("https://")
        )
    }
}
impl From<LoginDetails> for HomeView {
    fn from(login_details: LoginDetails) -> Self {
        Self {
            username: login_details.account.username,
            url: login_details.url,
            mastodon_client: login_details.mastodon_client,
            timeline: None,
        }
    }
}
impl HomeView {
    pub async fn run(&mut self, tx: mpsc::Sender<Event>) {
        match self.mastodon_client.get_home_timeline().await {
            Ok(timeline) => self.timeline = Some(timeline.initial_items),
            Err(e) => {
                if let Err(send_err) = tx.send(Event::MastodonError(e)).await {
                    eprintln!("Error sending MastodonError event: {}", send_err);
                }
            }
        }
        // simulate user activity
        sleep(Duration::from_secs(3)).await;
        if let Err(e) = tx.send(Event::LoggedOut).await {
            eprintln!("Error sending LoggedOut event: {}", e);
        }
    }

    pub(crate) fn widget(&self) -> List<'static> {
        let items = self.timeline.as_ref().map_or_else(
            || vec![ListItem::new("Loading...")],
            |timeline| {
                timeline
                    .iter()
                    .map(|status| ListItem::new(status.content.clone()))
                    .collect()
            },
        );
        List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(self.to_string()),
        )
    }
}
