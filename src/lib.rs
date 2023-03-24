use app::App;
use std::time::Duration;

pub mod app;
pub mod tui;
pub mod view;

const TICK_DURATION: Duration = Duration::from_millis(250);

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn run() -> crate::Result<()> {
    let app = App::build()?;
    app.run().await?;
    Ok(())
    // loop {
    //     tokio::select! {
    //         _ = app.run() => {
    //             println!("App exited");
    //             break;
    //         }
    //         _ = handle_crossterm_events(event_tx) => {
    //             println!("Crossterm event handler exited");
    //             break;
    //         }
    //     }
    // }
}
