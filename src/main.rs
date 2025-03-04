use app::App;
use color_eyre::eyre::Context;

mod app;
mod authentication;
mod config;
mod event;
mod home;
pub mod logging;
mod root;
mod widgets;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let (logs, _guard) = logging::init().wrap_err("logging init failed")?;
    let terminal = ratatui::init();
    let result = App::new(logs).run(terminal).await;
    ratatui::restore();
    result
}
