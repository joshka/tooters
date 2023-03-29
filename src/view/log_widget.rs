struct LogWidget {
    log_messages: Vec<String>,
}

impl Widget for LogWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().borders(Borders::ALL).title("Log");
        block.render(area, buf);
        let mut y = area.y + 1;
        for message in self.log_messages {
            buf.set_string(area.x + 1, y, message, Style::default());
            y += 1;
        }
    }
}

impl Display for LogWidget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Log")
    }
}
