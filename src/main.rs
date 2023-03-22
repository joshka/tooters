use tooters::app::{self, AppResult};

#[tokio::main]
async fn main() -> AppResult<()> {
    console_subscriber::init();
    app::run().await?;
    Ok(())
}
