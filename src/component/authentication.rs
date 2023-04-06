use crate::{config::Config, event::Event, widgets::AuthenticationWidget};
use anyhow::{Context, Result};
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use mastodon_async::{registration::Registered, Mastodon, Registration, StatusBuilder};
use ratatui::{backend::Backend, layout::Rect, Frame};
use std::sync::{Arc, Mutex};
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
    error: Arc<Mutex<Option<String>>>,
}

impl AuthenticationComponent {
    pub fn new(_app_event_sender: Sender<Event>) -> Self {
        Self {
            _app_event_sender,
            server_url_input: Input::new("https://mastodon.social".to_string()),
            server_url_sender: None,
            authenticated: false,
            error: Arc::new(Mutex::new(None)),
        }
    }

    pub fn title(&self) -> &'static str {
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
        let error = self.error.clone();
        tokio::spawn(async move {
            match run(rx).await {
                Ok(_) => info!("Authentication attempt finished"),
                Err(e) => {
                    error!("Authentication attempt failed: {:?}", e);
                    *error.lock().unwrap() = Some(e.to_string());
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
        let error = self.error.lock().unwrap().clone();
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
    if let Err(e) = Config::from(mastodon.data.clone()).save() {
        error!("Error saving config: {:?}", e);
    }
    let account = mastodon.verify_credentials().await?;
    info!("Logged in as {}", account.username);
    let status = StatusBuilder::new()
        .status("Hello from tooters!")
        .build()
        .unwrap();
    mastodon
        .new_status(status)
        .await
        .context("Error creating status")?;
    Ok(())
}

async fn authorize(rx: Receiver<String>) -> Result<Mastodon> {
    info!("Waiting for server url...");
    let server_url = get_server_url(rx).await?;
    info!("Registering Tooters at: {}", server_url);
    let registered = get_registered(server_url).await?;
    let (base, client_id, ..) = registered.clone().into_parts();
    info!("Tooters registered at: {} client_id: {}", base, client_id);
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
    let auth_code = webserver::get_code()
        .await
        .context("Error getting auth code from webserver")?;
    Ok(auth_code)
}

mod webserver {
    use axum::extract::Query;
    use axum::{extract::State, routing::get, Router};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::{mpsc, oneshot};
    use tracing::{error, info};

    pub async fn get_code() -> anyhow::Result<String> {
        let (tx, mut rx) = mpsc::channel(1);
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let server_handle = tokio::spawn(async {
            run_server(tx, shutdown_rx).await;
        });
        let code = rx.recv().await;
        shutdown_tx.send(()).unwrap_or_else(|e| {
            error!("Failed to send shutdown message to server: {:?}", e);
        });
        server_handle.await.unwrap_or_else(|e| {
            error!("Server task failed: {:?}", e);
        });
        Ok(code.unwrap())
    }

    async fn run_server(tx: mpsc::Sender<String>, shutdown_rx: oneshot::Receiver<()>) {
        let state = Arc::new(tx);
        let addr = ([127, 0, 0, 1], 7007).into();
        let router = Router::new()
            .route("/callback", get(handler))
            .with_state(state);
        info!("Listening on http://{addr}", addr = addr);
        axum::Server::bind(&addr)
            .serve(router.into_make_service())
            .with_graceful_shutdown(async {
                shutdown_rx.await.unwrap();
                info!("Shutting down server...")
            })
            .await
            .unwrap();
        info!("Server shutdown.")
    }

    async fn handler(
        State(state): State<Arc<mpsc::Sender<String>>>,
        Query(params): Query<HashMap<String, String>>,
    ) -> &'static str {
        if let Some(code) = params.get("code") {
            state
                .clone()
                .send(code.to_string())
                .await
                .unwrap_or_else(|e| {
                    error!("Failed to send code: {:?}", e);
                });
            "Authorized. You can close this window."
        } else {
            "Authorization failed. You can close this window."
        }
    }
}
