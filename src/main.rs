use tooters::{app::App, AppResult};

#[tokio::main]
async fn main() -> AppResult<()> {
    let app = App::new()?;
    app.run().await?;
    Ok(())
}
