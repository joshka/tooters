use crate::app::View;
use async_trait::async_trait;

pub struct LoggedOutView {
    reason: String,
}
impl LoggedOutView {
    pub fn new(reason: String) -> Self {
        Self { reason }
    }
}
#[async_trait]
impl View for LoggedOutView {
    fn title(&self) -> String {
        format!("Logged out. {}", self.reason)
    }
    async fn run(&mut self) {
        println!("Logged out. {}", self.reason)
    }
}
