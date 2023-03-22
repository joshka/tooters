use std::time::Duration;

use tokio::{time::sleep, sync::mpsc};

use crate::app::Event;

#[derive(Debug, Clone)]
pub struct HomeView {
    event_tx: mpsc::Sender<Event>,
}

impl HomeView {
    pub fn new(event_tx: mpsc::Sender<Event>) -> Self {
        Self { event_tx }
    }

    pub async fn run(self) {
        println!("HomeView::run");
        sleep(Duration::from_secs(1)).await;
        println!("HomeView::Pretend to do something");
        sleep(Duration::from_secs(1)).await;
        println!("HomeView::Pretend to do something");
        sleep(Duration::from_secs(1)).await;
        println!("HomeView::done");
        match self.event_tx.send(Event::LoggedOut).await {
            Ok(_) => println!("HomeView::LoggedOut"),
            Err(e) => println!("HomeView::LoggedOut error: {}", e),
        };
    }
}