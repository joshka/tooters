use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{tui::Tui, view::View, Event};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::Paragraph,
    Frame,
};
use tokio::{sync::mpsc, time::interval};

pub struct App {
    rx: mpsc::Receiver<Event>,
    tx: mpsc::Sender<Event>,
    tui: Tui,
    view: Arc<Mutex<View>>,
    messages: Vec<String>,
    tick_count: u64,
}

impl App {
    /// Build a new app.
    pub fn build() -> crate::Result<Self> {
        let (tx, rx) = mpsc::channel(100);
        let tui = Tui::build(tx.clone())?;
        Ok(Self {
            rx,
            tx,
            tui,
            view: Arc::new(Mutex::new(View::login())),
            messages: Vec::new(),
            tick_count: 0,
        })
    }

    /// Run the app.
    /// This will start the tick handler and handle events.
    pub async fn run(&mut self) -> crate::Result<()> {
        self.tui.init().await?;
        self.start_tick_handler();
        let tx = self.tx.clone();
        let view = self.view.clone();
        let view_task = tokio::spawn(async move {
            let mut view = view.lock().await;
            view.run(tx).await;
        });
        self.handle_events().await?;
        self.drain_events().await?;
        view_task.await?;
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

    /// Drain the event queue.
    /// This is used to ensure that all events are processed before exiting.
    async fn drain_events(&mut self) -> crate::Result<()> {
        self.rx.close();
        while (self.rx.recv().await).is_some() {}
        Ok(())
    }

    /// Handle events.
    /// This is the main event loop.
    async fn handle_events(&mut self) -> crate::Result<()> {
        while let Some(event) = self.rx.recv().await {
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
                    let home = View::home(login_details);
                    self.change_view(home).await;
                }
                Event::LoggedOut => {
                    self.messages.push("Logged out!".to_string());
                    let login = View::login();
                    self.change_view(login).await;
                }
                Event::Key(_event) => {}
                Event::MastodonError(err) => self.messages.push(err.to_string()),
            }
        }
        Ok(())
    }

    async fn change_view(&mut self, view: View) {
        let mut current_view = self.view.lock().await;
        *current_view = view;
        let tx = self.tx.clone();
        let view = self.view.clone();
        tokio::spawn(async move {
            let mut view = view.lock().await;
            view.run(tx).await;
        });
    }

    async fn draw(&mut self) -> crate::Result<()> {
        let view = self.view.lock().await;
        let view_title = view.to_string();
        self.tui.draw(|frame| {
            let size = frame.size();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // title bar
                    Constraint::Min(3),    // main view
                    Constraint::Length(1), // status bar
                ])
                .split(size);

            draw_title_bar(frame, layout[0], view_title);
            view.draw(frame, layout[1]);
            draw_status_bar(frame, layout[2], format!("Tick: {}", self.tick_count));
        })?;
        Ok(())
    }
}

fn draw_title_bar(frame: &mut Frame<impl Backend>, area: Rect, title: String) {
    let text = Spans::from(vec![
        Span::styled("Tooters", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(title, Style::default().fg(Color::Gray)),
    ]);
    let title_bar = Paragraph::new(text).style(Style::default().fg(Color::White).bg(Color::Blue));
    frame.render_widget(title_bar, area);
}

fn draw_status_bar(frame: &mut Frame<impl Backend>, area: Rect, text: String) {
    let text = Spans::from(vec![Span::raw(text)]);
    let status_bar = Paragraph::new(text).style(Style::default().fg(Color::White).bg(Color::Red));
    frame.render_widget(status_bar, area);
}
