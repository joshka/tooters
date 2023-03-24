use tooters::app;

#[tokio::main]
async fn main() -> tooters::Result<()> {
    console_subscriber::init();

    app::run().await?;

    Ok(())
}
