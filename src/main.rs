#[tokio::main]
async fn main() -> tooters::Result<()> {
    console_subscriber::init();
    tooters::run().await?;
    Ok(())
}
