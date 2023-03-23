use std::{time::Duration, fmt::Display};

use tokio::{time::sleep, sync::mpsc};

use crate::app::{Event, AppResult};

#[derive(Debug, Default)]
pub struct LoginView {
}

impl Display for LoginView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Login")
    }
}

impl LoginView {
    pub fn new() -> Self {
        Self {
        }
    }

    pub async fn run(&self, event_tx : mpsc::Sender<Event>) -> AppResult<()> {
        // simulate user login
        sleep(Duration::from_secs(1)).await;
        event_tx.send(Event::LoggedIn).await?;
        Ok(())
    }
}