use crate::event::Outcome;
use crate::{config::Config, event::Event};
use anyhow::{Context, Result};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use mastodon_async::{
    registration::Registered, scopes::Scopes, Mastodon, Registration, StatusBuilder,
};
use ratatui::{backend::Backend, layout::Rect, Frame};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Paragraph},
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tracing::{debug, error, info, trace, warn};
use tui_input::{backend::crossterm::EventHandler, Input};

#[derive(Debug)]
pub struct Authentication {
    _app_event_sender: Sender<Event>,
    server_url_input: Input,
    server_url_sender: Option<Sender<String>>,
    authenticated: bool,
    error: Arc<RwLock<Option<String>>>,
}

impl Authentication {
    pub fn new(app_event_sender: Sender<Event>) -> Self {
        Self {
            _app_event_sender: app_event_sender,
            server_url_input: Input::new("https://mastodon.social".to_string()),
            server_url_sender: None,
            authenticated: false,
            error: Arc::new(RwLock::new(None)),
        }
    }

    pub fn title(&self) -> String {
        String::from("Authenticating at ") + self.server_url_input.value()
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
    pub async fn handle_event(&mut self, event: &Event) -> Outcome {
        trace!(?event, "AuthenticationComponent::handle_event");
        match event {
            Event::Tick | Event::Quit => {}
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
        Outcome::NotConsumed
    }

    pub fn draw(&self, f: &mut Frame<impl Backend>, area: Rect) {
        let error = match self.error.read() {
            Ok(error) => error.clone(),
            Err(e) => {
                error!("Error locking error for read: {:?}", e);
                Some("Error locking error for read".to_string())
            }
        };
        let widget = Widget::new(error, self.server_url_input.value().to_string());
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
    let auth_code = get_code()
        .await
        .context("Error getting auth code from webserver")?;
    Ok(auth_code)
}

/// State for the axum webserver that allows the handler to send a code back
/// to the main thread and shutdown the webserver.
#[derive(Debug, Clone)]
struct AppState {
    code_sender: Sender<String>,
    shutdown_sender: Sender<()>,
}

/// Starts a webserver on port 7007 to listen for an authentication callback.
/// Returns the received authentication code when the callback is called.
async fn get_code() -> Result<String> {
    let port = 7007;
    let (code_sender, mut code_receiver) = channel::<String>(1);
    let (shutdown_sender, mut shutdown_reciever) = channel::<()>(1);
    let state = AppState {
        code_sender,
        shutdown_sender,
    };
    info!(
        "Starting webserver to listen for authentication callback on port {}",
        port
    );
    let addr = ([127, 0, 0, 1], port).into();
    let router = Router::new()
        .route("/callback", get(handler))
        .with_state(state);
    let server = axum::Server::bind(&addr).serve(router.into_make_service());
    server
        .with_graceful_shutdown(async {
            shutdown_reciever.recv().await;
        })
        .await
        .context("Error running webserver")?;
    code_receiver
        .recv()
        .await
        .context("Error receiving auth code from webserver")
}

/// Handles the `/callback` route for the webserver.
/// It extracts the authentication code from the query string and sends it to the main thread.
/// After that, it sends a shutdown signal to the webserver.
async fn handler(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> axum::response::Result<&'static str, AppError> {
    let code = params.get("code").context("No code in query string")?;
    state
        .code_sender
        .send(code.to_string())
        .await
        .context("Error sending code to main thread")?;
    state
        .shutdown_sender
        .send(())
        .await
        .context("Error sending shutdown signal to webserver")?;
    Ok("Authentication successful! You can close this window now.")
}

/// helper type to convert `anyhow::Error`s into responses
struct AppError(anyhow::Error);

/// Implements `IntoResponse` for `AppError`, converting it into a response with status code 500.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}

/// Implements the `From` trait for `AppError`, allowing it to be converted
/// from any type implementing `Into<anyhow::Error>`.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        Self(error.into())
    }
}

#[derive(Debug, Default)]
struct Widget {
    error: Option<String>,
    server_url: String,
}

impl Widget {
    pub const fn new(error: Option<String>, server_url: String) -> Self {
        Self { error, server_url }
    }

    pub fn draw(self, f: &mut Frame<impl Backend>, area: Rect) {
        f.render_widget(self, area);
    }
}

impl ratatui::widgets::Widget for Widget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let message_height = 10;
        let server_url_height = 3;

        if let [message_area, server_url_area] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(message_height),
                Constraint::Length(server_url_height),
            ])
            .split(area)
            .as_ref()
        {
            field(
                "Welcome to tooters. Sign in to your mastodon server",
                self.error.unwrap_or_default().as_str(),
            )
            .render(message_area, buf);

            field("Server URL:", &self.server_url).render(server_url_area, buf);
        }
    }
}

fn field<'a>(label: &'a str, value: &'a str) -> Paragraph<'a> {
    let title = Span::styled(label, Style::default().add_modifier(Modifier::BOLD));
    Paragraph::new(value).block(Block::default().title(title))
}
