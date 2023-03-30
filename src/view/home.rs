use std::fmt::Display;

use mastodon_async::{mastodon::Mastodon, prelude::Status};
use ratatui::{
    backend::Backend,
    layout::Rect,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use tokio::sync::mpsc;

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
    }

    pub fn draw(&self, frame: &mut Frame<impl Backend>, area: Rect) {
        let mut items = vec![];
        if let Some(timeline) = &self.timeline {
            for status in timeline {
                items.push(ListItem::new(format!(
                    "{}: {}",
                    status.account.display_name, status.content
                )));
            }
        } else {
            items.push(ListItem::new("Loading timeline..."));
        }
        let list = List::new(items).block(Block::default().borders(Borders::ALL).title("timeline"));
        frame.render_widget(list, area);
    }
}
