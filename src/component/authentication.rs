use crate::{authentication_server, config::Config, event::Event, widgets::AuthenticationWidget};
use anyhow::{Context, Result};
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use mastodon_async::{
    registration::Registered, scopes::Scopes, Mastodon, Registration, StatusBuilder,
};
use ratatui::{backend::Backend, layout::Rect, Frame};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{debug, error, info, trace, warn};
use tui_input::{backend::crossterm::EventHandler, Input};

use super::EventOutcome;

#[derive(Debug)]
pub struct AuthenticationComponent {
    _app_event_sender: Sender<Event>,
    server_url_input: Input,
    server_url_sender: Option<Sender<String>>,
    authenticated: bool,
    error: Arc<RwLock<Option<String>>>,
}

impl AuthenticationComponent {
    pub fn new(_app_event_sender: Sender<Event>) -> Self {
        Self {
            _app_event_sender,
            server_url_input: Input::new("https://mastodon.social".to_string()),
            server_url_sender: None,
            authenticated: false,
            error: Arc::new(RwLock::new(None)),
        }
    }

    pub const fn title(&self) -> &'static str {
        "Authentication"
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Starting authentication component");
        match load_config().await {
            Ok(_mastodon) => {
                self.authenticated = true;
                return Ok(());
            }
            Err(e) => {
                info!("Error loading config: {}", e);
            }
        };
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.server_url_sender = Some(tx);
        let error = Arc::clone(&self.error);
        tokio::spawn(async move {
            match run(rx).await {
                Ok(_) => info!("Authentication attempt finished"),
                Err(e) => {
                    error!("Authentication attempt failed: {:?}", e);
                    match error.write() {
                        Ok(mut error) => {
                            *error = Some(e.to_string());
                        }
                        Err(e) => {
                            error!("Error displaying error: {:?}", e);
                        }
                    }
                }
            }
        });
        Ok(())
    }
    pub async fn handle_event(&mut self, event: &Event) -> EventOutcome {
        trace!(?event, "AuthenticationComponent::handle_event");
        match event {
            Event::Tick => {}
            Event::Quit => {}
            Event::CrosstermEvent(CrosstermEvent::Key(key_event))
                if key_event.code == KeyCode::Enter =>
            {
                if let Some(tx) = &self.server_url_sender {
                    tx.clone()
                        .send(self.server_url_input.value().to_string())
                        .await
                        .ok();
                }
            }
            Event::CrosstermEvent(e) => {
                self.server_url_input.handle_event(e);
            }
        }
        EventOutcome::NotConsumed
    }

    pub fn draw(&self, f: &mut Frame<impl Backend>, area: Rect) {
        let error = match self.error.read() {
            Ok(error) => error.clone(),
            Err(e) => {
                error!("Error locking error for read: {:?}", e);
                Some("Error locking error for read".to_string())
            }
        };
        let widget = AuthenticationWidget::new(error, self.server_url_input.value().to_string());
        widget.draw(f, area);
    }
}

async fn load_config() -> Result<Mastodon> {
    let config = Config::load()?;
    info!("Loaded config");
    let mastodon = Mastodon::from(config.data);
    let account = mastodon.verify_credentials().await?;
    info!("Logged in as {}", account.username);
    Ok(mastodon)
}

async fn run(rx: Receiver<String>) -> Result<()> {
    info!("Running authentication flow");
    let mastodon = authorize(rx)
        .await
        .context("Error authorizing with mastodon")?;
    let config = Config::from(mastodon.data.clone());
    match config.save() {
        Ok(path) => info!("Saved config to {path}"),
        Err(e) => warn!("Error saving config: {}", e),
    }
    let account = mastodon.verify_credentials().await?;
    info!("Logged in as {}", account.username);
    send_test_toot(mastodon).await?;
    Ok(())
}

async fn send_test_toot(mastodon: Mastodon) -> Result<(), anyhow::Error> {
    let status = StatusBuilder::new()
        .status("Hello from tooters!")
        .build()
        .context("Unable to build test status")?;
    mastodon
        .new_status(status)
        .await
        .context("Unabled to send create status")?;
    info!("Tooted 'Hello from tooters' successfully!");
    Ok(())
}

async fn authorize(rx: Receiver<String>) -> Result<Mastodon> {
    info!("Waiting for server url...");
    let server_url = get_server_url(rx).await?;
    info!("Registering Tooters at: {}", server_url);
    let registered = get_registered(server_url).await?;
    info!("Tooters client registered");
    let auth_code = get_auth_code(&registered).await?;
    debug!("Auth code: {}", auth_code);
    let mastodon = get_mastodon(&registered, auth_code).await?;
    debug!("Mastodon: {:?}", mastodon);
    Ok(mastodon)
}

/// Get the server url from the user by asking them to enter it in the terminal
async fn get_server_url(mut rx: Receiver<String>) -> Result<String> {
    rx.recv()
        .await
        .ok_or_else(|| anyhow::Error::msg("Error getting server url"))
}

/// Register the client with the server
async fn get_registered(server_url: String) -> Result<Registered> {
    Registration::new(&server_url)
        .client_name("Tooters")
        .website("https://github.com/joshka/tooters")
        .redirect_uris("http://localhost:7007/callback")
        .scopes(Scopes::all())
        .build()
        .await
        .context(format!(
            "Error registering client with server: {server_url}"
        ))
}

async fn get_mastodon(registered: &Registered, code: String) -> Result<Mastodon> {
    registered
        .complete(code)
        .await
        .context("Unable to complete registration with the auth code")
}

/// Launch a browser to the authorization url and get the auth code from the user
/// Launch a server for the url redirect
async fn get_auth_code(registered: &Registered) -> Result<String> {
    let auth_url = registered
        .authorize_url()
        .context("Registered.authorize_url() is a result but it can't fail ¯\\_(ツ)_/¯")?;
    if webbrowser::open(&auth_url).is_ok() {
        info!("Opened browser to {}", auth_url);
    } else {
        warn!("Unable to open browser, please open this url: {}", auth_url);
    };
    let auth_code = authentication_server::get_code()
        .await
        .context("Error getting auth code from webserver")?;
    Ok(auth_code)
}
