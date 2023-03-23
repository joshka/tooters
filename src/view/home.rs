use std::{fmt::Display, time::Duration};

use tokio::{sync::mpsc, time::sleep};

use crate::app::{AppResult, Event};

#[derive(Debug, Default)]
pub struct HomeView {}

impl Display for HomeView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Home")
    }
}

impl HomeView {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&self, event_tx: mpsc::Sender<Event>) -> AppResult<()> {
        // simulate user logout
        sleep(Duration::from_secs(3)).await;
        event_tx.send(Event::LoggedOut).await?;
        Ok(())
    }
}
