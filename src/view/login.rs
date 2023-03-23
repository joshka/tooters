use std::{time::Duration, fmt::Display};

use tokio::{time::sleep, sync::mpsc};

use crate::app::{Event, AppResult};

pub struct LoginView {
    event_tx: mpsc::Sender<Event>,
}

impl Display for LoginView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Login")
    }
}

impl LoginView {
    pub fn new(event_tx : mpsc::Sender<Event>) -> Self {
        Self {
            event_tx,
        }
    }

    pub async fn run(&self) -> AppResult<()> {
        // simulate user login
        sleep(Duration::from_secs(1)).await;
        self.event_tx.send(Event::LoggedIn).await?;
        Ok(())
    }
}