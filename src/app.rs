use crossterm::event::{EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures::StreamExt;

use tokio::{sync::mpsc, time::interval};

use crate::{tui::Tui, view::login::LoginDetails, view::View};

#[derive(Debug)]
pub enum Event {
    Tick,
    Quit,
    Key(KeyEvent),
    LoggedIn(LoginDetails),
    LoggedOut,
}

pub struct App {
    rx: mpsc::Receiver<Event>,
    tx: mpsc::Sender<Event>,
    tui: Tui,
    tick_count: u64,
    title: String,
    view: View,
    next_view: Option<View>,
}

impl App {
    pub fn build() -> crate::Result<App> {
        let (tx, rx) = mpsc::channel(100);
        Ok(Self {
            rx,
            tx,
            tui: Tui::build()?,
            tick_count: 0,
            title: "".to_string(),
            view: View::None,
            next_view: Some(View::login()),
        })
    }

    pub async fn run(mut self) -> crate::Result<()> {
        self.tui.init()?;
        self.start().await?;
        self.handle_events().await?;
        self.drain_events().await?;
        Ok(())
    }

    async fn start(&self) -> crate::Result<()> {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let mut interval = interval(crate::TICK_DURATION);
            loop {
                interval.tick().await;
                if let Err(err) = tx.send(Event::Tick).await {
                    eprintln!("Error sending tick: {}", err);
                }
            }
        });
        Ok(())
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
                    if self.next_view.is_some() {
                        let view = self.next_view.take().unwrap();
                        self.view = view.clone();
                        self.title = view.to_string();
                        view.run(self.tx.clone()).await;
                    }
                    self.tick_count += 1;
                    // self.view.draw(&self.ui, &self.tick_count)?;
                }
                Event::Quit => {
                    break;
                }
                Event::LoggedIn(server) => {
                    self.next_view = Some(View::home(server));
                }
                Event::LoggedOut => {
                    self.next_view = Some(View::login());
                }
                Event::Key(_event) => {}
            }
        }
        Ok(())
    }
}
