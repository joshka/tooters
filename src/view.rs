use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Paragraph, Widget},
};
use std::fmt::Display;
use tokio::sync::mpsc;

use crate::{app::Event, tui::Tui};
use home::HomeView;
use login::LoginDetails;
use login::LoginView;

pub mod home;
pub mod login;

#[derive(Debug, Clone)]
pub enum View {
    Login(LoginView),
    Home(HomeView),
    None,
}

impl Display for View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            View::Login(view) => write!(f, "{}", view),
            View::Home(view) => write!(f, "{}", view),
            View::None => write!(f, "None"),
        }
    }
}

impl View {
    pub fn login() -> Self {
        Self::Login(LoginView::new())
    }

    pub fn home(login_details: LoginDetails) -> Self {
        Self::Home(HomeView::new(
            login_details.account.username,
            login_details.url,
            login_details.mastodon_client,
        ))
    }

    pub async fn run(self, event_tx: mpsc::Sender<Event>) {
        tokio::spawn(async move {
            let _result = match self {
                View::Login(view) => view.run(event_tx).await,
                View::Home(mut view) => view.run(event_tx).await,
                View::None => Ok(()),
            };
        });
    }

    pub fn widget(&self) -> Box<dyn Widget> {
        match self {
            View::Login(view) => Box::new(view.widget()),
            View::Home(view) => Box::new(view.widget()),
            View::None => Box::new(Paragraph::new("None")),
        }
    }

    pub fn draw(&self, ui: &Tui, tick_count: &u64) -> crate::Result<()> {
        let title = self.to_string();
        // ui.draw(|frame| {
        //     let size = frame.size();
        //     let layout = Layout::default()
        //         .direction(Direction::Vertical)
        //         .constraints([
        //             Constraint::Length(1),
        //             Constraint::Min(1),
        //             Constraint::Length(1),
        //         ])
        //         .split(size);

        //     let text = Spans::from(vec![
        //         Span::styled("Tooters", Style::default().add_modifier(Modifier::BOLD)),
        //         Span::raw(" | "),
        //         Span::styled(title, Style::default().fg(Color::Gray)),
        //     ]);
        //     let title_bar =
        //         Paragraph::new(text).style(Style::default().fg(Color::White).bg(Color::Blue));
        //     let tick_count = Paragraph::new(format!("Tick count: {}", tick_count));
        //     frame.render_widget(title_bar, layout[0]);
        //     frame.render_widget(tick_count, layout[2]);
        //     frame.render_widget(self, layout[1]);
        // })?;
        Ok(())
    }
}

// impl From<View> for dyn Widget {
//     fn from(view: View) -> Self {
//         match view {
//             View::Login(view) => view.widget().into(),
//             View::Home(view) => view.widget().into(),
//             View::None => Paragraph::new("None").into(),
//         }
//     }
// }

impl Widget for &View {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            View::Login(view) => view.widget().render(area, buf),
            View::Home(view) => view.widget().render(area, buf),
            View::None => Paragraph::new("None").render(area, buf),
        }
    }
}
