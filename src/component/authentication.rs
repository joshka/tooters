use std::sync::{Arc, RwLock};

use crate::event::Event;
use anyhow::{Context, Result};
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use mastodon_async::{
    helpers::toml,
    prelude::{Account, Status},
    registration::Registered,
    Data, Mastodon, Registration,
};
use ratatui::{
    backend::Backend,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Paragraph, Widget, Wrap},
    Frame,
};
use tokio::sync::{
    mpsc::{unbounded_channel, Sender, UnboundedReceiver, UnboundedSender},
    Mutex,
};
use tracing::{debug, error, info, trace, warn};
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{Component, EventOutcome};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
enum AuthenticationStep {
    #[default]
    LoadCredentialsFromFile,
    EnterServerUrl,
    RegisterClient,
    EnterAuthenticationCode,
    Authenticate,
    Done,
}

#[derive(Debug, Default)]
struct AuthenticationState {
    step: AuthenticationStep,
    server_url: Option<String>,
    auth_code: Option<String>,
    registered: Option<Registered>,
    mastodon: Option<Mastodon>,
    error: Option<String>, // mayber Option<Error> instead?
    toot: Option<Status>,
}

#[derive(Debug, PartialEq, Eq)]
enum AuthenticationEvent {
    UserEnteredServerUrl(String),
    UserEnteredAuthenticationCode(String),
    AuthenticationCodeCanceled,
}

#[derive(Debug)]
pub struct AuthenticationComponent {
    _app_event_sender: Sender<Event>,
    auth_state: Arc<RwLock<AuthenticationState>>,
    event_sender: UnboundedSender<AuthenticationEvent>,
    event_receiver: Arc<Mutex<UnboundedReceiver<AuthenticationEvent>>>,
    server_url_input: Input,
    auth_code_input: Input,
}

impl AuthenticationComponent {
    pub fn new(_app_event_sender: Sender<Event>) -> Self {
        let (event_sender, event_receiver) = unbounded_channel::<AuthenticationEvent>();
        let event_receiver = Arc::new(Mutex::new(event_receiver));
        let auth_state = Arc::new(RwLock::new(AuthenticationState::default()));
        Self {
            _app_event_sender,
            auth_state,
            event_sender,
            event_receiver,
            server_url_input: Input::new("https://mastodon.social".to_string()),
            auth_code_input: Input::default(),
        }
    }
}

impl Component for AuthenticationComponent {
    fn draw(&self, f: &mut Frame<impl Backend>, area: Rect) {
        f.render_widget(self, area);
        match self.auth_state.read().unwrap().step {
            AuthenticationStep::EnterServerUrl => {
                f.set_cursor(area.x + self.server_url_input.visual_cursor() as u16, area.y + 3);
            }
            AuthenticationStep::EnterAuthenticationCode => {
                f.set_cursor(
                    area.x + self.auth_code_input.visual_cursor() as u16,
                    area.y + 8,
                );
            }
            _ => {}
            // TODO show these to indicate that we're loading in case they take a while
            // AuthenticationStep::RegisterClient => todo!(),
            // AuthenticationStep::Authenticate => todo!(),
            // AuthenticationStep::Done => todo!(),
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventOutcome {
        trace!(?event, "AuthenticationComponent::handle_event");
        if event == &Event::Tick {
            return EventOutcome::Consumed;
        }
        // if in the EnterServerUrl or EnterAuthenticationCode state, we want to handle the event
        // by updating the appropriate input field unless the user presses Enter which should be
        // handled by sending an event to the event_sender
        // if in the RegisterClient or Authenticate state, we want to ignore the event
        match self.auth_state.read().unwrap().step {
            AuthenticationStep::LoadCredentialsFromFile
            | AuthenticationStep::RegisterClient
            | AuthenticationStep::Authenticate
            | AuthenticationStep::Done => {
                // don't handle events in these states (ignore keypresses etc.)
                EventOutcome::NotConsumed
            }
            AuthenticationStep::EnterServerUrl => match event {
                Event::CrosstermEvent(crossterm_event) => match crossterm_event {
                    CrosstermEvent::Key(key_event) => match key_event.code {
                        // let the root component handle this to allow the user to exit the app
                        KeyCode::Esc => EventOutcome::NotConsumed,
                        KeyCode::Enter => {
                            let url = self.server_url_input.value().to_string();
                            self.event_sender
                                .send(AuthenticationEvent::UserEnteredServerUrl(url))
                                .unwrap();
                            EventOutcome::Consumed
                        }
                        _ => {
                            self.server_url_input.handle_event(crossterm_event);
                            EventOutcome::Consumed
                        }
                    },
                    _ => EventOutcome::NotConsumed,
                },
                _ => EventOutcome::NotConsumed,
            },
            AuthenticationStep::EnterAuthenticationCode => match event {
                Event::CrosstermEvent(crossterm_event) => match crossterm_event {
                    CrosstermEvent::Key(key_event) => match key_event.code {
                        KeyCode::Esc => {
                            self.event_sender
                                .send(AuthenticationEvent::AuthenticationCodeCanceled)
                                .unwrap();
                            EventOutcome::Consumed
                        }
                        KeyCode::Enter => {
                            let auth_code = self.auth_code_input.value().to_string();
                            self.event_sender
                                .send(AuthenticationEvent::UserEnteredAuthenticationCode(
                                    auth_code,
                                ))
                                .unwrap();
                            EventOutcome::Consumed
                        }
                        _ => {
                            self.auth_code_input.handle_event(crossterm_event);
                            EventOutcome::Consumed
                        }
                    },
                    _ => EventOutcome::NotConsumed,
                },
                _ => EventOutcome::NotConsumed,
            },
        }
    }

    fn start(&mut self) {
        info!("Starting authentication component");
        let auth_state = Arc::clone(&self.auth_state);
        let event_receiver = Arc::clone(&self.event_receiver);
        tokio::spawn(async move {
            run_auth_flow(auth_state, event_receiver).await;
        });
    }
}

/// Run the authentication flow
/// This function is responsible for handling the authentication state machine
async fn run_auth_flow(
    auth_state: Arc<RwLock<AuthenticationState>>,
    event_receiver: Arc<Mutex<UnboundedReceiver<AuthenticationEvent>>>,
) {
    loop {
        let step = auth_state.read().unwrap().step;
        debug!(?step, "Authentication step");
        match step {
            AuthenticationStep::LoadCredentialsFromFile => {
                handle_load_credentials_state(&auth_state).await
            }
            AuthenticationStep::EnterServerUrl => {
                let mut event_receiver = event_receiver.lock().await;
                let event = event_receiver.recv().await;
                handle_enter_server_url_state(&auth_state, event).await
            }
            AuthenticationStep::RegisterClient => handle_register_client_state(&auth_state).await,
            AuthenticationStep::EnterAuthenticationCode => {
                let mut event_receiver = event_receiver.lock().await;
                let event = event_receiver.recv().await;
                handle_enter_authentication_code_state(&auth_state, event).await
            }
            AuthenticationStep::Authenticate => handle_authenticate_state(&auth_state).await,
            AuthenticationStep::Done => {
                handle_done_state(&auth_state).await;
                break;
            }
        }
    }
}

/// Handle the LoadCredentialsFromFile state
/// This state is entered when the app starts and attempts to load credentials from the credentials
/// file. If the credentials file exists and contains valid credentials, the app will skip the
/// authentication flow and go straight to the main app. If the credentials file doesn't exist or
/// contains invalid credentials, the app will enter the EnterServerUrl state.
async fn handle_load_credentials_state(auth_state: &Arc<RwLock<AuthenticationState>>) {
    let credentials = load_credentials();
    match credentials {
        Ok(mastodon) => {
            info!("Loaded credentials from file");
            let account = verify_credentials(mastodon.clone()).await;
            let mut auth_state = auth_state.write().unwrap();
            auth_state.mastodon = Some(mastodon);
            auth_state.error = Some(format!("Logged in as {}", account.unwrap().acct));
            auth_state.step = AuthenticationStep::Done;
        }
        Err(error) => {
            warn!(?error, "Failed to load credentials from file");
            let mut auth_state = auth_state.write().unwrap();
            auth_state.error = Some(format!(
                "Never logged in or problems loading saved credentials: {}",
                error
            ));
            auth_state.step = AuthenticationStep::EnterServerUrl;
        }
    }
}

/// Handle the EnterServerUrl state
/// This state is entered when the app needs to know the server URL to authenticate with. The user
/// will be prompted to enter the server URL and the app will enter the RegisterClient state.
/// If the user cancels the authentication flow, the app will exit.
async fn handle_enter_server_url_state(
    auth_state: &Arc<RwLock<AuthenticationState>>,
    event: Option<AuthenticationEvent>,
) {
    match event {
        Some(AuthenticationEvent::UserEnteredServerUrl(url)) => {
            debug!("User entered server url: {url}");
            let mut auth_state = auth_state.write().unwrap();
            auth_state.auth_code = None;
            auth_state.server_url = Some(url);
            auth_state.step = AuthenticationStep::RegisterClient;
        }
        event => {
            error!(?event, "Invalid event (expected UserEnteredServerUrl)");
            let mut auth_state = auth_state.write().unwrap();
            auth_state.error = Some(format!("Invalid event: {:?}", event));
        }
    }
}

/// Handle the RegisterClient state
/// This state is entered when the app needs to register a new client with the server. The app will
/// attempt to register a new client and enter the EnterAuthenticationCode state. If the app fails
/// to register a new client, the app will enter the EnterServerUrl state.
async fn handle_register_client_state(auth_state: &Arc<RwLock<AuthenticationState>>) {
    if auth_state.read().unwrap().server_url.is_none() {
        let mut auth_state = auth_state.write().unwrap();
        error!("No server URL entered");
        auth_state.error = Some("No server URL entered".to_string());
        auth_state.step = AuthenticationStep::EnterServerUrl;
        return;
    }
    let server_url = auth_state
        .read()
        .unwrap()
        .server_url
        .as_ref()
        .unwrap()
        .clone();
    // TODO we should allow the user to hit Escape to interrupt the registration process here
    // by using select! and checking the event_receiver
    match register_client(server_url).await {
        Ok(registered) => {
            debug!("Successfully registered client");
            let mut auth_state = auth_state.write().unwrap();
            auth_state.registered = Some(registered);
            auth_state.step = AuthenticationStep::EnterAuthenticationCode;
        }
        Err(err) => {
            error!(?err, "Failed to register client");
            let mut auth_state = auth_state.write().unwrap();
            auth_state.error = Some(format!("{:?}", err));
            auth_state.step = AuthenticationStep::EnterServerUrl;
        }
    }
}

/// Handle the EnterAuthenticationCode state
/// This state is entered when the app needs to know the authentication code to authenticate with.
/// The user will be prompted to enter the authentication code and the app will enter the
/// Authenticate state. If the user cancels the authentication flow, the app will enter the
/// EnterServerUrl state.
async fn handle_enter_authentication_code_state(
    auth_state: &Arc<RwLock<AuthenticationState>>,
    event: Option<AuthenticationEvent>,
) {
    match event {
        Some(AuthenticationEvent::UserEnteredAuthenticationCode(auth_code)) => {
            let mut auth_state = auth_state.write().unwrap();
            auth_state.auth_code = Some(auth_code);
            auth_state.step = AuthenticationStep::Authenticate;
        }
        Some(AuthenticationEvent::AuthenticationCodeCanceled) => {
            let mut auth_state = auth_state.write().unwrap();
            auth_state.step = AuthenticationStep::EnterServerUrl;
        }
        event => {
            error!(
                ?event,
                "Invalid event (expected AuthenticationCodeChanged or AuthenticationCodeCanceled)"
            );
            let mut auth_state = auth_state.write().unwrap();
            auth_state.error = Some(format!("Invalid event: {:?}", event));
            auth_state.step = AuthenticationStep::EnterServerUrl;
        }
    }
}

/// Handle the Authenticate state
/// This state is entered when the app needs to authenticate with the server. The app will attempt
/// to authenticate with the server and enter the Done state. If the app fails to authenticate with
/// the server, the app will enter the EnterServerUrl state.
async fn handle_authenticate_state(auth_state: &Arc<RwLock<AuthenticationState>>) {
    let registered = auth_state
        .read()
        .unwrap()
        .registered
        .as_ref()
        .unwrap()
        .clone();
    let auth_code = auth_state
        .read()
        .unwrap()
        .auth_code
        .as_ref()
        .unwrap()
        .clone();
    match authenticate(registered, auth_code).await {
        Ok(mastodon) => {
            debug!("Successfully authenticated with server");
            let mut auth_state = auth_state.write().unwrap();
            auth_state.mastodon = Some(mastodon);
            auth_state.step = AuthenticationStep::Done;
        }
        Err(err) => {
            error!("Error authenticating: {:?}", err);
            let mut auth_state = auth_state.write().unwrap();
            auth_state.error = Some(format!("{:?}", err));
            auth_state.step = AuthenticationStep::EnterServerUrl;
        }
    }
}

/// Handle the Done state
/// This state is entered when the app has successfully authenticated with the server.
///
async fn handle_done_state(auth_state: &Arc<RwLock<AuthenticationState>>) {
    debug!("Authentication done");
    let mastodon = auth_state.read().unwrap().mastodon.clone();

    let timeline = mastodon.unwrap().get_home_timeline().await.unwrap();
    debug!(?timeline, "Timeline");
    auth_state.write().unwrap().toot = Some(timeline.initial_items[0].clone());
    // TODO notify the main component that we're done with authentication
    let mastodon = auth_state
        .read()
        .unwrap()
        .mastodon
        .as_ref()
        .unwrap()
        .clone();
    let data = mastodon.data.clone();
    if let Err(err) = save_credentials(data) {
        error!(?err, "Failed to save credentials");
        auth_state.write().unwrap().error = Some(format!("Failed to save credentials: {:?}", err));
    } else {
        info!("Saved credentials");
    }
}

async fn register_client(server_url: String) -> Result<Registered> {
    Registration::new(&server_url)
        .client_name("tooters")
        .website("https://github.com/joshka/tooters")
        .build()
        .await
        .context(format!(
            "Error registering client with server: {server_url}"
        ))
}

async fn authenticate(registered: Registered, auth_code: String) -> Result<Mastodon> {
    registered
        .complete(auth_code)
        .await
        .context("Error authenticating with server")
}

/// Loads the config file from the XDG config directory
/// e.g. ~/.config/tooters/config.toml
fn load_credentials() -> Result<Mastodon> {
    let xdg = xdg::BaseDirectories::with_prefix("tooters")?;
    let config_file = xdg.get_config_file("config.toml");
    let data = toml::from_file(&config_file).with_context(|| {
        format!(
            "Unable to read config file from: {}",
            &config_file.to_string_lossy()
        )
    })?;
    Ok(Mastodon::from(data))
}

/// Saves the config file to the XDG config directory
/// e.g. ~/.config/tooters/config.toml
/// If the file already exists, it will be overwritten
/// If the directory does not exist, it will be created
fn save_credentials(data: Data) -> Result<()> {
    let xdg = xdg::BaseDirectories::with_prefix("tooters")?;
    let config_file = xdg.place_config_file("config.toml")?;
    toml::to_file(&data, &config_file).with_context(|| {
        format!(
            "Unable to write config file to: {}",
            &config_file.to_string_lossy()
        )
    })?;
    Ok(())
}

async fn verify_credentials(mastodon: Mastodon) -> Result<Account> {
    info!("Verifying credentials");
    let account = mastodon.verify_credentials().await?;
    info!(
        "Logged in as {} ({}) on {}",
        account.username, account.display_name, mastodon.data.base
    );
    Ok(account)
}

impl Widget for &AuthenticationComponent {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // 1. step
                Constraint::Length(1), // 2. error message
                Constraint::Length(1), // 3. server url label
                Constraint::Length(1), // 4. server url input
                Constraint::Length(1), // 5. authorization url label
                Constraint::Length(2), // 6. authorization url
                Constraint::Length(1), // 7. authentication code label
                Constraint::Length(1), // 8. authentication code input
                Constraint::Min(1),    // 9. empty space for a toot
            ])
            .split(area);

        let auth_state = self.auth_state.read().unwrap();
        let bold = Style::default().add_modifier(Modifier::BOLD);
        Paragraph::new(format!("Current Step: {:?}", auth_state.step))
            .style(bold)
            .render(layout[0], buf);
        if auth_state.error.is_some() {
            Paragraph::new(format!("Error: {}", auth_state.error.as_ref().unwrap()))
                .render(layout[1], buf);
        }
        Paragraph::new("Enter Server URL:")
            .style(bold)
            .render(layout[2], buf);
        Paragraph::new(self.server_url_input.value()).render(layout[3], buf);

        if auth_state.step == AuthenticationStep::EnterAuthenticationCode {
            Paragraph::new("Goto the following URL and enter the authentication code below:")
                .style(bold)
                .render(layout[4], buf);
            // woah!
            if let Ok(authorization_url) = self
                .auth_state
                .clone()
                .read()
                .unwrap()
                .registered
                .as_ref()
                .unwrap()
                .authorize_url()
            {
                Paragraph::new(authorization_url)
                    .wrap(Wrap { trim: true })
                    .render(layout[5], buf);
            }
            Paragraph::new("Authentication Code: ")
                .style(bold)
                .render(layout[6], buf);
            Paragraph::new(self.auth_code_input.value()).render(layout[7], buf);
        }

        if let Some(toot) = auth_state.toot.as_ref() {
            let html = toot.content.clone();
            let text = html2text::from_read(html.as_bytes(), area.width as usize);
            Paragraph::new(text).render(layout[8], buf);
        }
    }
}
