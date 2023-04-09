use parking_lot::Mutex;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use std::sync::Arc;

use crate::logging::LogMessage;

pub struct LogWidget {
    logs: Arc<Mutex<Vec<LogMessage>>>,
}

impl LogWidget {
    pub fn new(logs: Arc<Mutex<Vec<LogMessage>>>) -> Self {
        Self { logs }
    }
}

impl Widget for LogWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let logs = self.logs.lock();
        let max_lines = area.height as usize;
        let start_index = if logs.len() > max_lines {
            logs.len() - max_lines
        } else {
            0
        };

        let text = logs[start_index..]
            .iter()
            .map(|log| {
                let level_color = match log.level.as_str() {
                    "ERROR" => Color::Red,
                    "WARN" => Color::Yellow,
                    "INFO" => Color::Green,
                    "DEBUG" => Color::Blue,
                    "TRACE" => Color::Cyan,
                    _ => Color::White,
                };
                Spans::from(vec![
                    Span::styled(&log.level, Style::default().fg(level_color)),
                    Span::raw(" "),
                    Span::styled(&log.target, Style::default().add_modifier(Modifier::DIM)),
                    Span::styled(": ", Style::default().add_modifier(Modifier::DIM)),
                    Span::styled(&log.message, Style::default()),
                ])
            })
            .collect::<Vec<_>>();

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Logs"))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        paragraph.render(area, buf);
    }
}
