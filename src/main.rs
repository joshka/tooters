use app::App;
use color_eyre::eyre::Context;
use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
mod authentication;
mod config;
mod error;
mod event;
mod home;
pub mod logging;
mod root;
mod widgets;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    error::install_hooks()?;
    let (logs, _guard) = logging::init().wrap_err("logging init failed")?;
    let backend = CrosstermBackend::stdout().wrap_err("backend init failed")?;
    let terminal = Terminal::new(backend).wrap_err("terminal init failed")?;

    let app = App::new(logs);
    app.run(terminal).await?;
    Ok(())
}
