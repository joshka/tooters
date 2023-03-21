use std::time::{Duration, Instant};

use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::Paragraph,
};
use tokio::time::interval;

use crate::{ui::Ui, AppResult};

pub struct App {
    ui: Ui,
    start: Instant,
}

impl App {
    pub fn new() -> AppResult<Self> {
        let mut ui = Ui::new()?;
        ui.init()?;
        Ok(Self {
            ui,
            start: Instant::now(),
        })
    }

    pub async fn draw(&mut self) -> AppResult<()> {
        self.ui.draw(|frame| {
            let size = frame.size();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Min(1)])
                .split(size);
            let title_bar = Paragraph::new(Spans::from(vec![
                Span::styled("Tooters", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" | "),
                Span::styled("Press q to quit", Style::default().fg(Color::Gray)),
            ]))
            .style(Style::default().fg(Color::White).bg(Color::Blue));
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
                    if self.start.elapsed().as_secs() > 10 {
                        break;
                    }
                },
                event = events.next() => {
                    match event {
                        Some(Ok(Event::Key(key_event))) => {
                            if key_event.code == KeyCode::Char('q') {
                                break;
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
}
