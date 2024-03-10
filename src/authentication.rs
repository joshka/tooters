use crate::{config::Config, event::Event, event::Outcome};
use color_eyre::{eyre::Context, Result};
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use mastodon_async::{
    prelude::Account, registration::Registered, scopes::Scopes, Mastodon, Registration,
};
use parking_lot::RwLock;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    layout::{Constraint, Direction, Layout},
    style::Color,
    style::{Modifier, Style},
    text::Line,
    text::Span,
    widgets::Paragraph,
    Frame,
};
use std::sync::Arc;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    Mutex,
};
use tracing::{debug, error, info, trace, warn};
use tui_input::{backend::crossterm::EventHandler, Input};

#[derive(Debug)]
pub struct Authentication {
    event_sender: Sender<Event>,
    server_url_input: Input,
    server_url_sender: Sender<String>,
    server_url_receiver: Arc<Mutex<Receiver<String>>>,
    error: Arc<RwLock<Option<String>>>,
    authentication_data: Arc<RwLock<Option<State>>>,
}

#[derive(Debug, Clone)]
pub struct State {
    pub mastodon: Mastodon,
    pub config: Config,
    pub account: Account,
}

impl Authentication {
    pub fn new(
        event_sender: Sender<Event>,
        authentication_data: Arc<RwLock<Option<State>>>,
    ) -> Self {
        let (server_url_sender, server_url_receiver) = mpsc::channel(1);
        Self {
            event_sender,
            server_url_input: Input::new("https://mastodon.social".to_string()),
            server_url_sender,
            server_url_receiver: Arc::new(Mutex::new(server_url_receiver)),
            error: Arc::new(RwLock::new(None)),
            authentication_data,
        }
    }

    pub fn title(&self) -> String {
        String::from("Authenticating at ") + self.server_url_input.value()
    }

    pub async fn handle_event(&mut self, event: &Event) -> Outcome {
        trace!(?event, "AuthenticationComponent::handle_event");
        match event {
            Event::Crossterm(CrosstermEvent::Key(key_event))
                if key_event.code == KeyCode::Enter =>
            {
                self.server_url_sender
                    .clone()
                    .send(self.server_url_input.value().to_string())
                    .await
                    .ok();
                Outcome::Handled
            }
            Event::Crossterm(e) => {
                self.server_url_input.handle_event(e);
                Outcome::Handled
            }
            _ => Outcome::Ignored,
        }
    }

    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let error = self.error.read().clone();
        let widget = Widget::new(error, self.server_url_input.value().to_string());
        widget.draw(f, area);
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Starting authentication component");
        let error = Arc::clone(&self.error);
        let authentication_data = Arc::clone(&self.authentication_data);
        let server_url_receiver = self.server_url_receiver.clone();
        let event_sender = self.event_sender.clone();
        tokio::spawn(async move {
            loop {
                let server_url_receiver = server_url_receiver.clone();
                let authentication_data = authentication_data.clone();
                match load_config_or_authorize(server_url_receiver, authentication_data).await {
                    Ok(_) => break,
                    Err(e) => {
                        warn!("Authentication attempt failed: {:#}", e);
                        display_error(&e, &error);
                    }
                }
            }
            if let Err(err) = event_sender.send(Event::AuthenticationSuccess).await {
                error!("Error sending authentication success message: {:?}", err);
            }
        });
        Ok(())
    }
}

fn display_error(e: &color_eyre::eyre::Error, error: &Arc<RwLock<Option<String>>>) {
    *error.write() = Some(e.to_string());
}

async fn load_config_or_authorize(
    server_url_receiver: Arc<Mutex<Receiver<String>>>,
    authentication_data: Arc<RwLock<Option<State>>>,
) -> Result<()> {
    let (mastodon, config) = match Config::load() {
        Ok(config) => (Mastodon::from(config.data.clone()), config),
        Err(err) => {
            info!("Attempting authorization flow. {}", err);
            let mastodon = authorize(server_url_receiver)
                .await
                .context("unable to authorize")?;
            info!("Authorization successful");
            let config = Config::from(mastodon.data.clone());
            if let Err(err) = config.save() {
                // this is not fatal, but it means that we need to re-authenticate next time
                error!("Unable to save config file: {}", err);
            }
            (mastodon, config)
        }
    };

    let account = mastodon
        .verify_credentials()
        .await
        .context("failed to verify credentials")?;
    info!("Verified credentials. Logged in as {}", account.username);
    let mut authentication_data = authentication_data.write();
    *authentication_data = Some(State {
        mastodon: mastodon.clone(),
        config,
        account,
    });
    Ok(())
}

async fn authorize(server_url_receiver: Arc<Mutex<Receiver<String>>>) -> Result<Mastodon> {
    info!("Waiting for server url...");
    let server_url = get_server_url(server_url_receiver).await?;
    info!("Registering Toot-rs at: {}", server_url);
    let registered = register_client_app(server_url).await?;
    info!("Toot-rs client registered");
    let auth_code = get_auth_code(&registered).await?;
    debug!("Auth code: {}", auth_code);
    let mastodon = complete_registration(&registered, auth_code).await?;
    debug!("Mastodon: {:?}", mastodon);
    Ok(mastodon)
}

/// Get the server url from the user by asking them to enter it in the terminal
async fn get_server_url(server_url_receiver: Arc<Mutex<Receiver<String>>>) -> Result<String> {
    let mutex = server_url_receiver.clone();
    let mut server_url_receiver = mutex.lock().await;
    server_url_receiver
        .recv()
        .await
        .ok_or_else(|| color_eyre::eyre::Error::msg("Error getting server url"))
}

/// Register the client with the server
async fn register_client_app(server_url: String) -> Result<Registered> {
    Registration::new(&server_url)
        .client_name("Toot-rs")
        .website("https://github.com/toot-rs/toot-rs")
        .redirect_uris("http://localhost:7007/callback")
        .scopes(Scopes::all())
        .build()
        .await
        .context(format!("unable to register toot-rs with {server_url}"))
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
    let auth_code = server::get_code()
        .await
        .context("Error getting auth code from webserver")?;
    Ok(auth_code)
}

async fn complete_registration(registered: &Registered, code: String) -> Result<Mastodon> {
    registered
        .complete(code)
        .await
        .context("Unable to complete registration with the auth code")
}

/// a small webserver to listen for the authentication code callback from the
/// mastodon server
mod server {
    use axum::{
        extract::{Query, State},
        http::StatusCode,
        response::{IntoResponse, Response},
        routing::get,
        Router,
    };
    use color_eyre::{
        eyre::{Context, ContextCompat},
        Result,
    };
    use std::collections::HashMap;
    use tokio::sync::mpsc::{channel, Sender};
    use tracing::info;

    /// State for the axum webserver that allows the handler to send a code back
    /// to the main thread and shutdown the webserver.
    #[derive(Debug, Clone)]
    struct AppState {
        code_sender: Sender<String>,
        shutdown_sender: Sender<()>,
    }

    /// Starts a webserver on port 7007 to listen for an authentication callback.
    /// Returns the received authentication code when the callback is called.
    pub async fn get_code() -> Result<String> {
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
        let router = Router::new()
            .route("/callback", get(handler))
            .with_state(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:7007").await?;
        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
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

    /// helper type to convert `eyre::Error`s into responses
    struct AppError(color_eyre::eyre::Error);

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
        E: Into<color_eyre::eyre::Error>,
    {
        fn from(error: E) -> Self {
            Self(error.into())
        }
    }
} // mod server

#[derive(Debug, Default)]
struct Widget {
    error: Option<String>,
    server_url: String,
}

impl Widget {
    pub const fn new(error: Option<String>, server_url: String) -> Self {
        Self { error, server_url }
    }

    pub fn draw(self, f: &mut Frame, area: Rect) {
        f.render_widget(self, area);
    }
}

impl ratatui::widgets::Widget for Widget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let error_height = if self.error.is_some() { 2 } else { 0 };
        if let [welcome_area, error_area, server_url_area] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(error_height),
                Constraint::Length(2),
            ])
            .split(area)
            .as_ref()
        {
            Paragraph::new("Welcome to toot-rs. Sign in to your mastodon server.\nYou will be redirected to your browser to complete the authentication process.")
                .render(welcome_area, buf);

            if let Some(error) = self.error {
                Paragraph::new(Line::from(vec![
                    Span::styled(
                        "Error:",
                        Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
                    ),
                    Span::raw(" "),
                    Span::raw(error),
                ]))
                .render(error_area, buf);
            }

            Paragraph::new(Line::from(vec![
                Span::styled("Server URL:", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::raw(self.server_url),
            ]))
            .render(server_url_area, buf);
        }
    }
}
