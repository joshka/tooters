use std::fmt::Display;

use mastodon_async::{mastodon::Mastodon, prelude::Status};
use ratatui::widgets::{Block, Borders, List, ListItem};
use tokio::sync::mpsc;

use crate::app::{AppResult, Event};

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

impl HomeView {
    pub fn new(username: String, url: String, mastodon_client: Mastodon) -> Self {
        Self {
            username,
            url,
            mastodon_client,
            timeline: None,
        }
    }

    pub async fn run(&mut self, _event_tx: mpsc::Sender<Event>) -> AppResult<()> {
        let timeline = self.mastodon_client.get_home_timeline().await?;
        self.timeline = Some(timeline.initial_items);
        // sleep(Duration::from_secs(3)).await;
        // event_tx.send(Event::LoggedOut).await?;
        Ok(())
    }

    pub(crate) fn widget(&self) -> List<'static> {
        let items = match &self.timeline {
            None => vec![ListItem::new("Loading...")],
            Some(timeline) => timeline
                .iter()
                .map(|status| ListItem::new(status.content.clone()))
                .collect(),
        };
        List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(self.to_string()),
        )
    }
}
