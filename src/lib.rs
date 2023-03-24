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
}

use app::App;
use crossterm::event::KeyEvent;
use mastodon_async::prelude::Account;

#[derive(Debug)]
pub enum Event {
    Tick,
    Quit,
    Key(KeyEvent),
    LoggedIn(LoginDetails),
    LoggedOut,
    MastodonError(mastodon_async::Error),
}

#[derive(Debug, Clone)]
pub struct LoginDetails {
    pub url: String,
    pub account: Account,
    pub mastodon_client: mastodon_async::mastodon::Mastodon,
}
