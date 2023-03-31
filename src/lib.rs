use std::time::Duration;

pub mod app;
pub mod tui;
pub mod view;

const TICK_DURATION: Duration = Duration::from_millis(100);

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn run() -> crate::Result<()> {
    let mut app = App::build()?;
    app.run().await?;
    Ok(())
}

use app::App;
use crossterm::event::KeyEvent;
use view::login::LoginDetails;

#[derive(Debug)]
pub enum Event {
    Tick,
    Quit,
    Key(KeyEvent),
    LoggedIn(LoginDetails),
    LoggedOut,
    MastodonError(mastodon_async::Error),
}
