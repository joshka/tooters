use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Span, Spans};
use ratatui::widgets::{Paragraph, Widget};
use std::fmt::Display;
use tokio::sync::mpsc;

use crate::tui::Tui;
use crate::{Event, LoginDetails};
use home::HomeView;
use login::LoginView;

pub mod home;
pub mod login;

#[derive(Debug, Clone)]
pub enum View {
    Login(LoginView),
    Home(HomeView),
    // None,
}

impl Display for View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Login(view) => write!(f, "{view}"),
            Self::Home(view) => write!(f, "{view}"),
        }
    }
}

impl View {
    #[must_use]
    pub fn login() -> Self {
        Self::Login(LoginView::new())
    }

    #[must_use]
    pub fn home(login_details: LoginDetails) -> Self {
        Self::Home(HomeView::from(login_details))
    }

    pub async fn run(self, event_tx: mpsc::Sender<Event>) {
        tokio::spawn(async move {
            match self {
                Self::Login(view) => view.run(event_tx).await,
                Self::Home(mut view) => view.run(event_tx).await,
            };
        });
    }

    pub fn draw(&self, tui: &mut Tui, tick_count: u64) -> crate::Result<()> {
        let title = self.to_string();
        tui.draw(|frame| {
            let size = frame.size();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(3),
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
            //     let tick_count = Paragraph::new(format!("Tick count: {}", tick_count));
            frame.render_widget(title_bar, layout[0]);
            frame.render_widget(self, layout[1]);

            // let items = errors.iter().map(|e| ListItem::new(e.to_string())).collect::<Vec<_>>();
            // let widget = List::new(items).block(Block::default().borders(Borders::ALL).title("Errors"));
            // status bar with th tick count
            let text = Spans::from(vec![Span::raw(format!("Tick count: {tick_count}"))]);
            let widget = Paragraph::new(text).style(Style::default().bg(Color::Red));
            frame.render_widget(widget, layout[2]);
        })?;
        Ok(())
    }
}

impl Widget for &View {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            View::Login(view) => view.widget().render(area, buf),
            View::Home(view) => view.widget().render(area, buf),
        };
    }
}
