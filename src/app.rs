use crate::{tui::Tui, view::View, Event};
use tokio::{sync::mpsc, time::interval};

pub struct App {
    rx: mpsc::Receiver<Event>,
    tx: mpsc::Sender<Event>,
    tui: Tui,
    view: Option<View>,
    messages: Vec<String>,
    tick_count: u64,
}

impl App {
    pub fn build() -> crate::Result<Self> {
        let (tx, rx) = mpsc::channel(100);
        let tui = Tui::build(tx.clone())?;
        Ok(Self {
            rx,
            tx,
            tui,
            view: Some(View::login()),
            messages: Vec::new(),
            tick_count: 0,
        })
    }

    pub async fn run(mut self) -> crate::Result<()> {
        self.tui.init().await?;
        self.start_tick_handler();
        self.handle_events().await?;
        self.drain_events().await?;
        Ok(())
    }

    /// Start a tick handler that sends a tick event every `TICK_DURATION`.
    /// This is used to update the UI.
    fn start_tick_handler(&self) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let mut interval = interval(crate::TICK_DURATION);
            loop {
                interval.tick().await;
                if tx.send(Event::Tick).await.is_err() {
                    break;
                }
            }
        });
    }

    async fn drain_events(&mut self) -> crate::Result<()> {
        self.rx.close();
        while (self.rx.recv().await).is_some() {}
        Ok(())
    }

    async fn handle_events(&mut self) -> crate::Result<()> {
        while let Some(event) = self.rx.recv().await {
            match event {
                Event::Tick => {
                    self.tick_count += 1;
                    if let Some(view) = &self.view {
                        view.draw(&mut self.tui, self.tick_count)?;
                    }
                }
                Event::Quit => {
                    break;
                }
                Event::LoggedIn(login_details) => {
                    self.messages.push("Logged in!".to_string());
                    let tx = self.tx.clone();
                    let view = self.view.insert(View::home(login_details.clone()));
                    if let View::Home(home) = view {
                        home.run(tx).await;
                    }
                }
                Event::LoggedOut => {
                    self.messages.push("Logged out!".to_string());
                    self.view = Some(View::login());
                }
                Event::Key(_event) => {}
                Event::MastodonError(err) => self.messages.push(err.to_string()),
            }
        }
        Ok(())
    }
}
