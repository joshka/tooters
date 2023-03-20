use std::time::{Duration, Instant};

use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use ratatui::{
    layout::{Alignment::Center, Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
};
use tokio::time::interval;

use crate::{ui::Ui, AppResult};

pub struct App {
    ui: Ui,
    title: &'static str,
    start: Instant,
}

impl App {
    pub fn new() -> AppResult<Self> {
        let mut ui = Ui::new()?;
        ui.init()?;
        Ok(Self {
            ui,
            title: "Tooters",
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
            let block = Block::default()
                .borders(Borders::TOP)
                .title_alignment(Center)
                .title(self.title);
            frame.render_widget(block, layout[0]);
            let output = Paragraph::new(format!("Elapsed: {:?}", self.start.elapsed().as_millis()))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title_alignment(Center)
                        .title("Output"),
                );
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
