use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Paragraph, Widget},
};

#[derive(Debug, Default)]
pub struct TitleBar {
    title: &'static str,
}

impl TitleBar {
    pub const HEIGHT: u16 = 1;
    pub const fn new(title: &'static str) -> Self {
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
