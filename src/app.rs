use crossterm::event::{KeyCode, KeyModifiers};
use std::sync::Arc;
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{
    tui::Tui,
    view::{home::HomeView, login::LoginView, View},
    Event,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Paragraph, Widget},
};
use tokio::{sync::mpsc, time::interval};

pub struct App {
    event_receiver: mpsc::Receiver<Event>,
    event_sender: mpsc::Sender<Event>,
    tui: Tui,
    view: Arc<Mutex<View>>,
    messages: Vec<String>,
    tick_count: u64,
}

impl App {
    /// Build a new app.
    pub fn build() -> crate::Result<Self> {
        let (event_sender, event_receiver) = mpsc::channel(100);
        let tui = Tui::build(event_sender.clone())?;
        Ok(Self {
            event_receiver,
            event_sender,
            tui,
            view: Arc::new(Mutex::new(View::Login(LoginView::new()))),
            messages: Vec::new(),
            tick_count: 0,
        })
    }

    /// Run the app.
    /// This will start the tick handler and handle events.
    pub async fn run(&mut self) -> crate::Result<()> {
        self.tui.init().await?;
        self.start_tick_handler();
        let event_sender = self.event_sender.clone();
        let view = self.view.clone();
        let view_task = tokio::spawn(async move {
            let mut view = view.lock().await;
            view.run(event_sender).await.unwrap_or_default()
        });
        self.handle_events().await?;
        self.drain_events().await?;
        view_task.await?;
        Ok(())
    }

    /// Start a tick handler that sends a tick event every `TICK_DURATION`.
    /// This is used to update the UI.
    fn start_tick_handler(&self) {
        let event_sender = self.event_sender.clone();
        tokio::spawn(async move {
            let mut interval = interval(crate::TICK_DURATION);
            loop {
                interval.tick().await;
                if event_sender.send(Event::Tick).await.is_err() {
                    break;
                }
            }
        });
    }

    /// Drain the event queue.
    /// This is used to ensure that all events are processed before exiting.
    async fn drain_events(&mut self) -> crate::Result<()> {
        self.event_receiver.close();
        while (self.event_receiver.recv().await).is_some() {}
        Ok(())
    }

    /// Handle events.
    /// This is the main event loop.
    async fn handle_events(&mut self) -> crate::Result<()> {
        while let Some(event) = self.event_receiver.recv().await {
            match event {
                Event::Tick => {
                    self.tick_count += 1;
                    self.draw().await?;
                }
                Event::Quit => {
                    break;
                }
                Event::LoggedIn(login_details) => {
                    self.messages.push("Logged in!".to_string());
                    self.change_view(View::Home(HomeView::from(login_details)))
                        .await;
                }
                Event::LoggedOut => {
                    self.messages.push("Logged out!".to_string());
                    self.change_view(View::Login(LoginView::new())).await;
                }
                Event::Key(key) => {
                    let mut view = self.view.lock().await;
                    if let View::Home(ref mut home_view) = *view {
                        match (key.modifiers, key.code) {
                            (KeyModifiers::NONE, KeyCode::Char('j')) => {
                                home_view.scroll_down();
                            }
                            (KeyModifiers::NONE, KeyCode::Char('k')) => {
                                home_view.scroll_up();
                            }
                            _ => {}
                        }
                    }
                }
                Event::MastodonError(_err) => {}
            }
        }
        Ok(())
    }

    async fn change_view(&mut self, view: View) -> JoinHandle<()> {
        let mut current_view = self.view.lock().await;
        *current_view = view;
        let event_sender = self.event_sender.clone();
        let view = self.view.clone();
        tokio::spawn(async move {
            let mut view = view.lock().await;
            view.run(event_sender).await.unwrap_or_default()
        })
    }

    async fn draw(&mut self) -> crate::Result<()> {
        let view = self.view.lock().await;
        let view_title = view.title();
        let view_status = view.status();
        self.tui.draw(|frame| {
            let size = frame.size();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(TitleBar::HEIGHT),
                    Constraint::Min(0),
                    Constraint::Length(StatusBar::HEIGHT),
                ])
                .split(size);

            frame.render_widget(TitleBar::new(view_title), layout[0]);
            view.draw(frame, layout[1]);
            frame.render_widget(
                // render a tick count here to show that the UI is updating
                // StatusBar::new(format!("Tick: {}", self.tick_count)),
                StatusBar::new(view_status),
                layout[2],
            );
        })?;
        Ok(())
    }
}

struct TitleBar {
    title: String,
}

impl TitleBar {
    const HEIGHT: u16 = 1;
    const fn new(title: String) -> Self {
        Self { title }
    }
}

impl Widget for TitleBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default().fg(Color::White).bg(Color::Blue);
        let bold = Style::default().add_modifier(Modifier::BOLD);
        let gray = Style::default().fg(Color::Gray);
        let text = Spans::from(vec![
            Span::styled("Tooters", bold),
            Span::raw(" | "),
            Span::styled(self.title, gray),
        ]);
        Paragraph::new(text).style(style).render(area, buf);
    }
}

struct StatusBar {
    text: String,
}

impl StatusBar {
    const HEIGHT: u16 = 1;
    const fn new(text: String) -> Self {
        Self { text }
    }
}

impl Widget for StatusBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default().fg(Color::White).bg(Color::Blue);
        let bold = Style::default().add_modifier(Modifier::BOLD);
        let text = Span::raw(self.text);
        let text = Spans::from(vec![
            Span::styled("Q", bold),
            Span::raw("uit | "),
            Span::styled("J", bold),
            Span::raw(" down | "),
            Span::styled("K", bold),
            Span::raw(" up | "),
            text,
        ]);
        Paragraph::new(text).style(style).render(area, buf);
    }
}
