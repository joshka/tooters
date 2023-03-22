use crossterm::event::{KeyEvent, EventStream, KeyCode, KeyModifiers};
use futures::StreamExt;
use ratatui::{layout::{Direction, Constraint, Layout}, style::{Modifier, Style, Color}, text::{Spans, Span}, widgets::Paragraph};
use tokio::{sync::mpsc, time::interval};

use crate::{view::{LoginView, View}, ui::Ui};

pub async fn run() -> AppResult<()> {
    let mut app = App::new()?;
    crossterm::terminal::enable_raw_mode()?;
    let mut key_events = EventStream::new();
    loop {
        tokio::select! {
            _ = app.run() => {
                println!("App exited");
                break;
            }
            Some(Ok(crossterm::event::Event::Key(key_event))) = key_events.next() => {
                match (key_event.modifiers, key_event.code) {
                    (crossterm::event::KeyModifiers::CONTROL, KeyCode::Char('c')) |
                    (KeyModifiers::NONE, KeyCode::Char('q'))
                    => {
                        app.tx.send(Event::Quit).await?;
                    }
                    _ => {
                        app.tx.send(Event::Key(key_event)).await?;
                    }
                }
            }
        }
    }
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub enum Event {
    Tick,
    Start,
    Quit,
    Key(KeyEvent),
    LoggedIn,
    LoggedOut,
}

pub struct App {
    rx: mpsc::Receiver<Event>,
    tx: mpsc::Sender<Event>,
    ui: Ui,
    title: String,
}

impl Default for App {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl App {
    pub fn new() -> AppResult<App> {
        let (tx, rx) = mpsc::channel(100);
        let mut ui = Ui::new()?;
        ui.init()?;
        Ok(Self { rx, tx, ui, title: "".to_string() })
    }

    pub async fn run(&mut self) -> AppResult<()> {
        self.start().await?;
        self.handle_events().await?;
        self.drain_events().await?;
        Ok(())
    }

    async fn start(&self) -> AppResult<()> {
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
        self.tx.send(Event::Start).await?;
        Ok(())
    }

    async fn drain_events(&mut self) -> AppResult<()> {
        self.rx.close();
        while (self.rx.recv().await).is_some() {}
        Ok(())
    }

    async fn handle_events(&mut self) -> AppResult<()> {
        while let Some(event) = self.rx.recv().await {
            match event {
                Event::Tick => {
                    self.draw()?;
                }
                Event::Quit => {
                    break;
                }
                Event::Start => {
                    let view = View::Login(LoginView::new(self.tx.clone()));
                    self.title = "Login".to_string();
                    view.run().await;
                }
                Event::LoggedOut => {
                }
                Event::LoggedIn => {
                }
                Event::Key(_event) => {
                },
            }
        }
        Ok(())
    }

    fn draw(&mut self) -> AppResult<()> {
        let title = &self.title;
        self.ui.draw(|frame| {
            let size = frame.size();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(1),
                    Constraint::Length(1),
                ])
                .split(size);

            let text = Spans::from(vec![
                Span::styled("Tooters", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" | "),
                Span::styled(title, Style::default().fg(Color::Gray)),
            ]);
            let title_bar =
                Paragraph::new(text).style(Style::default().fg(Color::White).bg(Color::Blue));
            frame.render_widget(title_bar, layout[0]);
        })?;
        Ok(())
    }
}
