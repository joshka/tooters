use std::{time::Duration, fmt::Display};

use tokio::{time::sleep, sync::mpsc};

use crate::app::{Event, AppResult};

pub struct HomeView {
    event_tx: mpsc::Sender<Event>,
}

impl Display for HomeView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Home")
    }
}

impl HomeView {
    pub fn new(event_tx: mpsc::Sender<Event>) -> Self {
        Self { event_tx }
    }

    pub async fn run(&self) -> AppResult<()> {
        // simulate user logout
        sleep(Duration::from_secs(3)).await;
        self.event_tx.send(Event::LoggedOut).await?;
        Ok(())
    }
}