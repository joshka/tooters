#[tokio::main]
async fn main() -> tooters::Result<()> {
    tooters::run().await?;
    Ok(())
}
