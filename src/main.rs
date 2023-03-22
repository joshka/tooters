use tooters::{app::App, AppResult};

#[tokio::main]
async fn main() -> AppResult<()> {
    // femme::with_level(log::LevelFilter::Trace);
    let app = App::new()?;
    app.run().await?;
    Ok(())
}
