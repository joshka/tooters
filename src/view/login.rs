use mastodon_async::{
    helpers::toml, prelude::Account, registration::Registered, Data, Registration,
};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Paragraph,
    Frame,
};
use tokio::sync::mpsc;
use tui_input::Input;

use crate::Event;

#[derive(Debug, Clone)]
pub struct LoginDetails {
    pub url: String,
    pub account: Account,
    pub mastodon_client: mastodon_async::mastodon::Mastodon,
}

#[derive(Debug, Default)]
pub struct LoginView {
    state: LoginState,
    status: &'static str,
    input: Input,
    flash_message: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub enum LoginState {
    InputServerUrl,
    RegisteringApp(String),
    InputAuthenticationCode(Registered),
    #[default]
    LoadingCredentialsFile,
    VerifyingCredentials(Data),
    LoggedIn(LoginDetails),
    CompleteAuth(Registered, String),
}

impl LoginView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(&self) -> String {
        "Login".to_string()
    }

    pub async fn run(&mut self, event_tx: mpsc::Sender<Event>) -> crate::Result<()> {
        loop {
            self.state = match &self.state {
                LoginState::LoadingCredentialsFile => {
                    self.status = "Loading credentials file...";
                    match self.load_credentials().await {
                        Ok(data) => LoginState::VerifyingCredentials(data),
                        Err(e) => {
                            self.flash_message =
                                Some(format!("Error loading credentials file: {}", e));
                            LoginState::InputServerUrl
                        }
                    }
                }
                LoginState::VerifyingCredentials(data) => {
                    self.status = "Verifying credentials...";
                    match self.verify_credentials(data.clone()).await {
                        Ok(login_details) => LoginState::LoggedIn(login_details),
                        Err(e) => {
                            self.flash_message =
                                Some(format!("Error verifying credentials: {}", e));
                            LoginState::InputServerUrl
                        }
                    }
                }
                LoginState::LoggedIn(login_details) => {
                    self.status = "Logged in";
                    event_tx
                        .send(Event::LoggedIn(login_details.clone()))
                        .await?;
                    break;
                }
                LoginState::InputServerUrl => {
                    self.status = "Enter server url";
                    match self.read_server_url().await {
                        Ok(url) => LoginState::RegisteringApp(url),
                        Err(e) => {
                            self.flash_message = Some(format!("Error reading server url: {}", e));
                            LoginState::InputServerUrl
                        }
                    }
                }
                LoginState::RegisteringApp(url) => {
                    self.status = "Registering app...";
                    match self.register_app(url.clone()).await {
                        Ok(registered) => LoginState::InputAuthenticationCode(registered),
                        Err(e) => {
                            self.flash_message = Some(format!("Error registering app: {}", e));
                            LoginState::InputServerUrl
                        }
                    }
                }
                LoginState::InputAuthenticationCode(registered) => {
                    self.status = "Enter authentication code";
                    match self.read_authentication_code().await {
                        Ok(code) => LoginState::CompleteAuth(registered.clone(), code),
                        Err(e) => {
                            self.flash_message =
                                Some(format!("Error reading authorization code: {}", e));
                            LoginState::InputServerUrl
                        }
                    }
                }
                LoginState::CompleteAuth(registered, code) => {
                    registered.complete(code.clone()).await?;
                    todo!("save credentials")
                }
            };
        }
        Ok(())
    }

    async fn load_credentials(&self) -> crate::Result<Data> {
        toml::from_file("mastodon-data.toml").map_err(|e| e.into())
    }

    pub fn status(&self) -> String {
        self.status.into()
    }

    pub fn draw(&self, frame: &mut Frame<impl Backend>, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(area);

        let message = self.flash_message.as_deref().unwrap_or("");
        let message_widget = Paragraph::new(message);
        frame.render_widget(message_widget, layout[0]);

        let input_widget = Paragraph::new(self.input.value());
        frame.render_widget(input_widget, layout[1]);
    }

    pub fn handle_event(&mut self, event: Event) {
        if let Event::Key(_key) = event {
            // Update the input state with the key event
            // self.input.handle_event(event);

            // Process the key event and update the LoginView state accordingly.
            // For example, if the user presses a specific key to initiate the registration process:
            // if key == /* the specific key for registration */ {
            // Call the process_registration method of the LoginView.
            // }
            // If the user is entering the response code during the authorization process:
            // Update the state of the LoginView with the entered response code.
        }
    }

    async fn verify_credentials(&self, data: Data) -> crate::Result<LoginDetails> {
        let url = data.base.to_string();
        let mastodon_client = mastodon_async::mastodon::Mastodon::from(data.clone());
        let account = mastodon_client.verify_credentials().await?;
        let login_details = LoginDetails {
            url,
            account,
            mastodon_client,
        };
        Ok(login_details)
    }

    async fn read_server_url(&self) -> crate::Result<String> {
        // todo actually prompt for server
        let url = "https://mastodon.social".to_string();
        Ok(url)
    }

    async fn register_app(&self, url: String) -> crate::Result<Registered> {
        let registration = Registration::new(url)
            .client_name("tooters")
            // .redirect_uris("urn:ietf:wg:oauth:2.0:oob")
            // .scopes(Scopes::read_all())
            .website("https://github.com/joshka/tooters")
            .build()
            .await?;
        Ok(registration)
    }

    async fn read_authentication_code(&self) -> crate::Result<String> {
        // todo actually prompt for code
        let code = "asdf".to_string();
        Ok(code)
    }
}
