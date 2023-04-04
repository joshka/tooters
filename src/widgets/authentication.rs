use ratatui::{
    backend::Backend,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Paragraph, Widget},
    Frame,
};

#[derive(Debug, Default)]
pub struct AuthenticationWidget {
    error: Option<String>,
    server_url: String,
}

impl AuthenticationWidget {
    pub fn new(error: Option<String>, server_url: String) -> Self {
        Self { error, server_url }
    }

    pub fn draw(self, f: &mut Frame<impl Backend>, area: Rect) {
        f.render_widget(self, area);
    }
}

impl Widget for AuthenticationWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let message_height = 3;
        let server_url_height = 3;

        if let [message_area, server_url_area] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(message_height),
                Constraint::Length(server_url_height),
            ])
            .split(area)
            .as_ref()
        {
            field(
                "Welcome to tooters. Sign in to your mastodon server",
                self.error.unwrap_or_default().as_str(),
            )
            .render(message_area, buf);

            field("Server URL:", &self.server_url).render(server_url_area, buf);
        }
    }
}

fn field<'a>(label: &'a str, value: &'a str) -> Paragraph<'a> {
    let title = Span::styled(label, Style::default().add_modifier(Modifier::BOLD));
    Paragraph::new(value).block(Block::default().title(title))
}
