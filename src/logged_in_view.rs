use crate::app::View;
use async_trait::async_trait;

pub struct LoggedInView {
    username: String,
}
impl LoggedInView {
    pub fn new(username: String) -> Self {
        Self { username }
    }
}
#[async_trait]
impl View for LoggedInView {
    fn title(&self) -> String {
        format!("Logged in as @{}", self.username)
    }
    async fn run(self) {
        
    }
}
