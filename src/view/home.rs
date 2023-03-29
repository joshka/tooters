use std::{fmt::Display, time::Duration};

use mastodon_async::{mastodon::Mastodon, prelude::Status};
use ratatui::widgets::{Block, Borders, List, ListItem, Widget};
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
}

impl Widget for HomeView {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let mut items = vec![];
        if let Some(timeline) = &self.timeline {
            for status in timeline {
                items.push(ListItem::new(format!(
                    "{}: {}",
                    status.account.display_name, status.content
                )));
            }
        }
        List::new(items)
            .block(Block::default().borders(Borders::ALL).title("timeline"))
            .render(area, buf);
    }
}
