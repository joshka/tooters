use std::time::Duration;

use tokio::{time::sleep, sync::mpsc};

use crate::app::Event;

#[derive(Debug, Clone)]
pub struct LoginView {
    event_tx: mpsc::Sender<Event>,
}

impl LoginView {
    pub fn new(event_tx : mpsc::Sender<Event>) -> Self {
        Self {
            event_tx,
        }
    }

    pub async fn run(self) {
        println!("LoginView::run");
        sleep(Duration::from_secs(1)).await;
        println!("Pretend to login");
        sleep(Duration::from_secs(1)).await;
        println!("LoginView::done");
        match self.event_tx.send(Event::LoggedIn).await {
            Ok(_) => println!("LoginView::LoggedIn"),
            Err(e) => println!("LoginView::LoggedIn error: {}", e),
        };
    }
}