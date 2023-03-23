use crossterm::event::{EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::Paragraph,
};
use tokio::{sync::mpsc, time::interval};

use crate::{ui::Ui, view::View};

pub async fn run() -> AppResult<()> {
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

pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub enum Event {
    Tick,
    Quit,
    Key(KeyEvent),
    LoggedIn,
    LoggedOut,
}

pub struct App {
    rx: mpsc::Receiver<Event>,
    tx: mpsc::Sender<Event>,
    ui: Ui,
    tick_count: u64,
    title: String,
    next_view: Option<View>,
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
        Ok(Self {
            rx,
            tx,
            ui,
            tick_count: 0,
            title: "".to_string(),
            next_view: Some(View::login()),
        })
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
                    if self.next_view.is_some() {
                        let view = self.next_view.take().unwrap();
                        self.title = view.to_string();
                        view.run(self.tx.clone()).await;
                    }
                    self.tick_count += 1;
                    self.draw()?;
                }
                Event::Quit => {
                    break;
                }
                Event::LoggedIn => {
                    self.next_view = Some(View::home());
                }
                Event::LoggedOut => {
                    self.next_view = Some(View::login());
                }
                Event::Key(_event) => {}
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
            let tick_count = Paragraph::new(format!("Tick count: {}", self.tick_count));
            frame.render_widget(title_bar, layout[0]);
            frame.render_widget(tick_count, layout[2]);
        })?;
        Ok(())
    }
}
