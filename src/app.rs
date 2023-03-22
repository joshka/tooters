use std::time::{Duration, Instant};

use async_trait::async_trait;
use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::Paragraph,
};
use tokio::{sync::mpsc, time::interval};

use crate::{initial_view::IntitialView, ui::Ui, AppResult, logged_in_view::LoggedInView};

pub struct App {
    ui: Ui,
    start: Instant,
    current_view: Box<dyn View>,
    rx: mpsc::Receiver<AppEvent>,
    tx: mpsc::Sender<AppEvent>,
}

impl App {
    pub fn new() -> AppResult<Self> {
        let (tx, rx) = mpsc::channel(100);
        let mut ui = Ui::new()?;
        ui.init()?;
        Ok(Self {
            ui,
            start: Instant::now(),
            current_view: Box::new(IntitialView::new(tx.clone())),
            rx,
            tx,
        })
    }

    pub async fn draw(&mut self) -> AppResult<()> {
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

            let title = self.current_view.title();
            let text = Spans::from(vec![
                Span::styled("Tooters", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" | "),
                Span::styled(title, Style::default().fg(Color::Gray)),
            ]);
            let title_bar =
                Paragraph::new(text).style(Style::default().fg(Color::White).bg(Color::Blue));
            frame.render_widget(title_bar, layout[0]);

            let output = Paragraph::new(format!(
                "Elapsed: {:?} millis",
                self.start.elapsed().as_millis()
            ));
            frame.render_widget(output, layout[1]);
        })?;
        Ok(())
    }

    pub async fn run(mut self) -> AppResult<()> {
        let mut events = EventStream::new();
        let mut interval = interval(Duration::from_millis(1000));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.draw().await?;
                },
                event = events.next() => {
                    match event {
                        Some(Ok(Event::Key(key_event))) => {
                            if key_event.code == KeyCode::Char('q') {
                                self.tx.send(AppEvent::Quit).await?;
                            }
                        },
                        _ => {}
                    }
                },
                event = self.rx.recv() => {
                    match event {
                        Some(AppEvent::Quit) => break,
                        Some(AppEvent::LoggedIn(username)) => {
                            self.current_view = Box::new(LoggedInView::new(username));
                        },
                        Some(AppEvent::LoggedOut(_message)) => {
                            // self.current_view = Box::new(LoggedOutView::new(message));
                        },
                        None => break,
                    }
                }

            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum AppEvent {
    Quit,
    LoggedIn(String),
    LoggedOut(String),
}

#[async_trait]
pub(crate) trait View {
    fn title(&self) -> String;
    async fn run(self);
}
