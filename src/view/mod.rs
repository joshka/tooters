mod login;
pub use login::LoginView;

mod home;
pub use home::HomeView;

#[derive(Debug, Clone)]
pub enum View {
    Login(LoginView),
    Home(HomeView),
    None,
}

impl View {
    pub async fn run(self) {
        tokio::spawn(async move {
            match self {
                View::Login(view) => view.run().await,
                View::Home(view) => view.run().await,
                View::None => {},
            }
        });
    }
}