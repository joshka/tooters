use crossterm::event::{EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures::StreamExt;

use tokio::{sync::mpsc, time::interval};

use crate::{ui::Ui, view::login::LoginDetails, view::View};

pub async fn run() -> crate::Result<()> {
    let mut app = App::new()?;
    crossterm::terminal::enable_raw_mode()?;
    loop {
        let event_tx = app.tx.clone();
        tokio::select! {
            _ = app.run() => {
                println!("App exited");
                break;
            }
            _ = handle_crossterm_events(event_tx) => {
                println!("Crossterm event handler exited");
                break;
            }
        }
    }
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

pub async fn handle_crossterm_events(event_tx: mpsc::Sender<Event>) {
    let mut key_events = EventStream::new();
    while let Some(Ok(crossterm::event::Event::Key(key_event))) = key_events.next().await {
        match (key_event.modifiers, key_event.code) {
            (crossterm::event::KeyModifiers::CONTROL, KeyCode::Char('c'))
            | (KeyModifiers::NONE, KeyCode::Char('q')) => {
                event_tx.send(Event::Quit).await.unwrap();
            }
            _ => {
                event_tx.send(Event::Key(key_event)).await.unwrap();
            }
        }
    }
}

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
    ui: Ui,
    tick_count: u64,
    title: String,
    view: View,
    next_view: Option<View>,
}

impl Default for App {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl App {
    pub fn new() -> crate::Result<App> {
        let (tx, rx) = mpsc::channel(100);
        let mut ui = Ui::new()?;
        ui.init()?;
        Ok(Self {
            rx,
            tx,
            ui,
            tick_count: 0,
            title: "".to_string(),
            view: View::None,
            next_view: Some(View::login()),
        })
    }

    pub async fn run(&mut self) -> crate::Result<()> {
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
