use std::time::Duration;

use tokio::{time::sleep, sync::mpsc};

use crate::app::{Event, AppResult};

pub struct HomeView {
    event_tx: mpsc::Sender<Event>,
}

impl HomeView {
    pub fn new(event_tx: mpsc::Sender<Event>) -> Self {
        Self { event_tx }
    }

    pub async fn run(self) -> AppResult<()> {
        // simulate user logout
        sleep(Duration::from_secs(3)).await;
        self.event_tx.send(Event::LoggedOut).await?;
        Ok(())
    }
}