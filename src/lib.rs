use std::time::Duration;

pub mod app;
pub mod ui;
pub mod view;

const TICK_DURATION: Duration = Duration::from_millis(250);

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn run() -> Result<()> {
    Ok(())
}
