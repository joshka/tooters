use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use std::sync::{Arc, Mutex};

pub struct LogWidget {
    logs: Arc<Mutex<Vec<String>>>,
}

impl LogWidget {
    pub fn new(logs: Arc<Mutex<Vec<String>>>) -> Self {
        Self { logs }
    }
}

impl Widget for LogWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let logs = self.logs.lock().unwrap();
        let max_lines = area.height as usize;
        let start_index = if logs.len() > max_lines {
            logs.len() - max_lines
        } else {
            0
        };

        let text = logs[start_index..]
            .iter()
            .map(|log| Spans::from(Span::styled(log.clone(), Style::default().fg(Color::White))))
            .collect::<Vec<_>>();

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Logs"))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        paragraph.render(area, buf);
    }
}
