use std::time::Duration;

use tokio::{time::sleep, sync::mpsc};

use crate::app::{Event, AppResult};

pub struct LoginView {
    event_tx: mpsc::Sender<Event>,
}

impl LoginView {
    pub fn new(event_tx : mpsc::Sender<Event>) -> Self {
        Self {
            event_tx,
        }
    }

    pub async fn run(self) -> AppResult<()> {
        // simulate user login
        sleep(Duration::from_secs(1)).await;
        self.event_tx.send(Event::LoggedIn).await?;
        Ok(())
    }
}